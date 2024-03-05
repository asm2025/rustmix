use core::time;
use crossbeam::channel;
use std::{
    error::Error,
    future::Future,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc, Condvar, Mutex,
    },
    thread,
    time::Duration,
};

pub enum TaskResult<T> {
    None,
    Cancelled,
    TimedOut,
    Error(T, String),
    Success(T),
}

pub trait TaskDelegate<T: Send + Sync + Clone + 'static> {
    fn on_task(&self, pc: &ProducerConsumer<T>, item: T) -> Result<TaskResult<T>, Box<dyn Error>>;

    fn on_result(&self, pc: &ProducerConsumer<T>, result: TaskResult<T>) -> bool;
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProducerConsumerOptions {
    pub capacity: usize,
    pub threshold: Duration,
    pub sleep_after_enqueue: Duration,
}

impl ProducerConsumerOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(self, capacity: usize) -> Self {
        ProducerConsumerOptions { capacity, ..self }
    }

    pub fn with_threshold(self, threshold: Duration) -> Self {
        ProducerConsumerOptions { threshold, ..self }
    }

    pub fn with_sleep_after_enqueue(self, sleep_after_enqueue: Duration) -> Self {
        ProducerConsumerOptions {
            sleep_after_enqueue,
            ..self
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProducerConsumer<T: Send + Sync + Clone + 'static> {
    options: ProducerConsumerOptions,
    queue: Arc<Mutex<Vec<T>>>,
    condvar: Arc<Condvar>,
    completed: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    peek_timeout: Duration,
    producers: Arc<AtomicUsize>,
    consumers: Arc<AtomicUsize>,
    running: Arc<AtomicUsize>,
    sender: channel::Sender<T>,
    receiver: channel::Receiver<T>,
}

impl<T: Send + Sync + Clone> ProducerConsumer<T> {
    pub fn new() -> Self {
        let options: ProducerConsumerOptions = Default::default();
        let (sender, receiver) = if options.capacity > 0 {
            channel::bounded::<T>(options.capacity)
        } else {
            channel::unbounded::<T>()
        };
        ProducerConsumer {
            options: options,
            sender: sender,
            receiver: receiver,
            queue: Arc::new(Mutex::new(Vec::new())),
            condvar: Arc::new(Condvar::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            peek_timeout: time::Duration::from_millis(10),
            producers: Arc::new(AtomicUsize::new(0)),
            consumers: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn with_options(options: ProducerConsumerOptions) -> Self {
        let (sender, receiver) = if options.capacity > 0 {
            channel::bounded::<T>(options.capacity)
        } else {
            channel::unbounded::<T>()
        };
        ProducerConsumer {
            options: options,
            sender: sender,
            receiver: receiver,
            queue: Arc::new(Mutex::new(Vec::new())),
            condvar: Arc::new(Condvar::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            peek_timeout: time::Duration::from_millis(10),
            producers: Arc::new(AtomicUsize::new(0)),
            consumers: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn options(&self) -> &ProducerConsumerOptions {
        &self.options
    }

    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    pub fn is_completed(&self) -> bool {
        self.completed.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_busy(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::Relaxed) > 0
    }

    pub fn count(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    pub fn producers(&self) -> usize {
        self.producers.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn inc_producers(&self) {
        self.producers
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn dec_producers(&self) {
        self.producers
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn consumers(&self) -> usize {
        self.consumers.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn inc_consumers(&self) {
        self.consumers
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn dec_consumers(&self) {
        self.consumers
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn running(&self) -> usize {
        self.running.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn inc_running(&self) {
        self.running
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn dec_running(&self) {
        self.running
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn start_producer(self) {
        self.inc_producers();
        let prod = Arc::new(Mutex::new(self.clone()));
        thread::spawn(move || {
            let producer = prod.lock().unwrap();

            loop {
                let Some(item) = producer.pop() else {
                    break;
                };

                if producer.is_cancelled() || producer.is_completed() {
                    break;
                }

                producer.inc_running();
                producer.sender.send(item).unwrap();
                thread::sleep(producer.options.sleep_after_enqueue);
            }

            producer.dec_producers();
        });
    }

    pub fn start_consumer<S: TaskDelegate<T> + Send + Sync + Clone + 'static>(
        &self,
        task_delegate: S,
    ) {
        self.inc_consumers();
        let cons = Arc::new(Mutex::new(self.clone()));
        let td = task_delegate.clone();
        thread::spawn(move || {
            let consumer = cons.lock().unwrap();

            loop {
                if consumer.is_cancelled() || (consumer.is_empty() && consumer.is_completed()) {
                    break;
                }

                let Ok(item) = consumer.receiver.recv_timeout(consumer.peek_timeout) else {
                    continue;
                };

                if let Ok(result) = td.on_task(&consumer, item) {
                    if !td.on_result(&consumer, result) {
                        consumer.dec_running();
                        break;
                    }

                    consumer.dec_running();
                }
            }

            consumer.dec_consumers();
        });
    }

    pub fn stop(&self, enforce: bool) {
        if enforce {
            self.cancel();
        } else {
            self.complete();
        }
    }

    pub fn enqueue(&self, item: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(item);
        self.condvar.notify_one();
    }

    fn pop(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();

        while queue.is_empty() && !self.is_cancelled() && !self.is_completed() {
            let result = self.condvar.wait_timeout(queue, self.peek_timeout).unwrap();
            queue = result.0;

            if result.1.timed_out() {
                thread::sleep(Duration::ZERO);
                continue;
            }

            if self.is_cancelled() || self.is_completed() {
                return None;
            }

            return queue.pop();
        }

        if queue.is_empty() || self.is_cancelled() {
            return None;
        }

        queue.pop()
    }

    pub fn dequeue(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();

        if queue.is_empty() {
            return None;
        }

        queue.pop()
    }

    pub fn peek(&self) -> Option<T> {
        let queue = self.queue.lock().unwrap();

        if queue.is_empty() {
            return None;
        }

        Some(queue[0].clone())
    }

    pub fn clear(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
    }

    pub fn complete(&self) {
        self.completed
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.condvar.notify_all();
    }

    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.condvar.notify_all();
    }

    pub fn pause(&self) {
        self.paused
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.paused
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.condvar.notify_all();
    }

    pub fn wait(&self) -> bool {
        todo!()
    }

    pub fn wait_async(&self) -> Box<dyn Future<Output = bool>> {
        todo!()
    }

    pub fn wait_for(&self, timeout: Duration) -> bool {
        todo!()
    }

    pub fn wait_for_async(&self, timeout: Duration) -> Box<dyn Future<Output = bool>> {
        todo!()
    }
}
