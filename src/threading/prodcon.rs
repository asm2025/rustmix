use crossbeam::channel;
use std::{
    error::Error,
    future::Future,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc, Condvar, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tokio::sync::Notify;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskResult {
    #[default]
    None,
    Cancelled,
    TimedOut,
    Error(String),
    Success,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QueueBehavior {
    #[default]
    FIFO,
    LIFO,
}

pub trait TaskDelegate<T: Send + Sync + Clone + 'static> {
    fn on_task(&self, pc: &ProducerConsumer<T>, item: T) -> Result<TaskResult, Box<dyn Error>>;
    fn on_result(&self, pc: &ProducerConsumer<T>, item: T, result: TaskResult) -> bool;
    fn on_finished(&self, pc: &ProducerConsumer<T>);
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProducerConsumerOptions {
    pub capacity: usize,
    pub behavior: QueueBehavior,
    pub threshold: Duration,
    pub sleep_after_enqueue: Duration,
    peek_timeout: Duration,
    pause_timeout: Duration,
}

impl Default for ProducerConsumerOptions {
    fn default() -> Self {
        ProducerConsumerOptions {
            capacity: 0,
            behavior: QueueBehavior::FIFO,
            threshold: Duration::ZERO,
            sleep_after_enqueue: Duration::ZERO,
            peek_timeout: Duration::from_millis(10),
            pause_timeout: Duration::from_millis(50),
        }
    }
}

impl ProducerConsumerOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(self, capacity: usize) -> Self {
        ProducerConsumerOptions { capacity, ..self }
    }

    pub fn with_behavior(self, behavior: QueueBehavior) -> Self {
        ProducerConsumerOptions { behavior, ..self }
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

    pub fn peek_timeout(&self) -> Duration {
        self.peek_timeout
    }

    pub fn set_peek_timeout(&mut self, value: Duration) {
        self.peek_timeout = if value.is_zero() {
            Duration::from_millis(10)
        } else if value.as_secs() > 1 {
            Duration::from_secs(1)
        } else {
            value
        };
    }

    pub fn pause_timeout(&self) -> Duration {
        self.pause_timeout
    }

    pub fn set_pause_timeout(&mut self, value: Duration) {
        self.pause_timeout = if value.is_zero() {
            Duration::from_millis(50)
        } else if value.as_secs() > 1 {
            Duration::from_secs(1)
        } else {
            value
        };
    }
}

#[derive(Clone, Debug)]
pub struct ProducerConsumer<T: Send + Sync + Clone + 'static> {
    options: ProducerConsumerOptions,
    queue: Arc<Mutex<Vec<T>>>,
    queue_cond: Arc<Condvar>,
    finished: Arc<Mutex<bool>>,
    finished_cond: Arc<Condvar>,
    finished_notify: Arc<Notify>,
    completed: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    producers_count: Arc<AtomicUsize>,
    consumers_count: Arc<AtomicUsize>,
    running_count: Arc<AtomicUsize>,
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
            queue_cond: Arc::new(Condvar::new()),
            finished: Arc::new(Mutex::new(false)),
            finished_cond: Arc::new(Condvar::new()),
            finished_notify: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            producers_count: Arc::new(AtomicUsize::new(0)),
            consumers_count: Arc::new(AtomicUsize::new(0)),
            running_count: Arc::new(AtomicUsize::new(0)),
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
            queue_cond: Arc::new(Condvar::new()),
            finished: Arc::new(Mutex::new(false)),
            finished_cond: Arc::new(Condvar::new()),
            finished_notify: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            producers_count: Arc::new(AtomicUsize::new(0)),
            consumers_count: Arc::new(AtomicUsize::new(0)),
            running_count: Arc::new(AtomicUsize::new(0)),
        }
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
        (self
            .producers_count
            .load(std::sync::atomic::Ordering::Relaxed)
            > 0)
            || (self
                .consumers_count
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0)
            || (self
                .running_count
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0)
            || (self.queue.lock().unwrap().len() > 0)
    }

    pub fn count(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    pub fn producers(&self) -> usize {
        self.producers_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn inc_producers(&self) {
        self.producers_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn dec_producers(&self, td: &dyn TaskDelegate<T>) {
        self.producers_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        self.check_finished(td);
    }

    pub fn consumers(&self) -> usize {
        self.consumers_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn inc_consumers(&self) {
        self.consumers_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn dec_consumers(&self, td: &dyn TaskDelegate<T>) {
        self.consumers_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        self.check_finished(td);
    }

    fn check_finished(&self, td: &dyn TaskDelegate<T>) {
        if self.is_completed() && self.producers() == 0 && self.consumers() == 0 {
            let mut finished = self.finished.lock().unwrap();
            *finished = true;
            td.on_finished(self);
            self.finished_cond.notify_all();
            self.finished_notify.notify_waiters();
        }
    }

    pub fn running(&self) -> usize {
        self.running_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn inc_running(&self) {
        self.running_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn dec_running(&self) {
        self.running_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn start_producer<S: TaskDelegate<T> + Send + Sync + Clone + 'static>(
        &self,
        task_delegate: S,
    ) {
        self.inc_producers();
        let prod = Arc::new(Mutex::new(self.clone()));
        let td = task_delegate.clone();
        let builder = thread::Builder::new().name(format!("Producer {}", self.producers()));
        builder
            .spawn(move || {
                let producer = prod.lock().unwrap();

                loop {
                    if producer.is_cancelled() || producer.is_completed() {
                        break;
                    }

                    if producer.is_paused() {
                        thread::sleep(producer.options.pause_timeout);
                        continue;
                    }

                    if let Some(item) = match producer.options.behavior {
                        QueueBehavior::FIFO => producer.dequeue(),
                        QueueBehavior::LIFO => producer.pop(),
                    } {
                        if producer.is_cancelled() || producer.is_completed() {
                            break;
                        }

                        producer.inc_running();
                        producer.sender.send(item).unwrap();

                        if !producer.options.sleep_after_enqueue.is_zero() {
                            thread::sleep(producer.options.sleep_after_enqueue);
                        }
                    };
                }

                if !producer.is_cancelled() {
                    while let Some(item) = match producer.options.behavior {
                        QueueBehavior::FIFO => producer.dequeue(),
                        QueueBehavior::LIFO => producer.pop(),
                    } {
                        if producer.is_cancelled() {
                            break;
                        }

                        if producer.is_paused() {
                            thread::sleep(producer.options.pause_timeout);
                            continue;
                        }

                        producer.inc_running();
                        producer.sender.send(item).unwrap();

                        if !producer.options.sleep_after_enqueue.is_zero() {
                            thread::sleep(producer.options.sleep_after_enqueue);
                        }
                    }
                }

                producer.dec_producers(&td);
            })
            .unwrap();
    }

    pub fn start_consumer<S: TaskDelegate<T> + Send + Sync + Clone + 'static>(
        &self,
        task_delegate: S,
    ) {
        self.inc_consumers();
        let cons = Arc::new(Mutex::new(self.clone()));
        let td = task_delegate.clone();
        let builder = thread::Builder::new().name(format!("Consumer {}", self.consumers()));
        builder
            .spawn(move || {
                let consumer = cons.lock().unwrap();

                loop {
                    if consumer.is_cancelled()
                        || (consumer.is_empty()
                            && consumer.is_completed()
                            && consumer.running() == 0)
                    {
                        break;
                    }

                    if consumer.is_paused() {
                        thread::sleep(consumer.options.pause_timeout);
                        continue;
                    }

                    let Ok(item) = consumer
                        .receiver
                        .recv_timeout(consumer.options.peek_timeout)
                    else {
                        continue;
                    };

                    if let Ok(result) = td.on_task(&consumer, item.clone()) {
                        if !td.on_result(&consumer, item, result) {
                            consumer.dec_running();
                            break;
                        }

                        consumer.dec_running();
                    }
                }

                consumer.dec_consumers(&td);
            })
            .unwrap();
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
        self.queue_cond.notify_one();
    }

    fn dequeue(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();

        while queue.is_empty() && !self.is_cancelled() && !self.is_completed() {
            let result = self
                .queue_cond
                .wait_timeout(queue, self.options.peek_timeout)
                .unwrap();
            queue = result.0;

            if result.1.timed_out() {
                thread::sleep(Duration::ZERO);
                continue;
            }

            if self.is_cancelled() || self.is_completed() {
                return None;
            }

            return Some(queue.remove(0));
        }

        if queue.is_empty() || self.is_cancelled() {
            return None;
        }

        Some(queue.remove(0))
    }

    fn pop(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();

        while queue.is_empty() && !self.is_cancelled() && !self.is_completed() {
            let result = self
                .queue_cond
                .wait_timeout(queue, self.options.peek_timeout)
                .unwrap();
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

    pub fn remove(&self, index: usize) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();

        if queue.is_empty() || index >= queue.len() {
            return None;
        }

        Some(queue.remove(index))
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
        self.queue_cond.notify_all();
    }

    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.queue_cond.notify_all();
    }

    pub fn pause(&self) {
        self.paused
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.paused
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.queue_cond.notify_all();
    }

    pub fn wait(&self) {
        let finished = self.finished.lock().unwrap();

        if !*finished {
            let _ignored = self.finished_cond.wait(finished).unwrap();
        }
    }

    pub async fn wait_async(&self) {
        while !*self.finished.lock().unwrap() {
            self.finished_notify.notified().await;
            thread::sleep(self.options.peek_timeout);
        }
    }

    pub fn wait_for(&self, timeout: Duration) -> bool {
        if timeout == Duration::ZERO {
            self.wait();
            return true;
        }

        let start = Instant::now();
        let mut finished = self.finished.lock().unwrap();

        while !*finished && start.elapsed() < timeout {
            let result = self
                .finished_cond
                .wait_timeout(finished, self.options.peek_timeout)
                .unwrap();
            finished = result.0;
            thread::sleep(self.options.peek_timeout);

            if result.1.timed_out() || start.elapsed() >= timeout {
                return false;
            }
        }

        start.elapsed() < timeout
    }

    pub async fn wait_for_async(&self, timeout: Duration) -> Box<dyn Future<Output = bool>> {
        if timeout == Duration::ZERO {
            self.wait_async().await;
            return Box::new(async { true });
        }

        let start = Instant::now();
        let mut finished = self.finished.lock().unwrap();

        while !*finished && start.elapsed() < timeout {
            let result = self
                .finished_cond
                .wait_timeout(finished, self.options.peek_timeout)
                .unwrap();
            finished = result.0;
            thread::sleep(self.options.peek_timeout);

            if result.1.timed_out() || start.elapsed() >= timeout {
                return Box::new(async move { false });
            }
        }

        Box::new(async move { start.elapsed() < timeout })
    }
}
