use crossbeam::channel;
use std::{
    error::Error,
    future::Future,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Condvar, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tokio::sync::Notify;

use super::*;

pub trait ProducerConsumerDelegation<T: Send + Clone + 'static> {
    fn process(&self, pc: &ProducerConsumer<T>, item: &T) -> Result<TaskResult, Box<dyn Error>>;
    fn on_completed(&self, pc: &ProducerConsumer<T>, item: &T, result: TaskResult) -> bool;
    fn on_finished(&self, pc: &ProducerConsumer<T>);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProducerConsumerOptions {
    pub capacity: usize,
    pub threshold: Duration,
    pub sleep_after_send: Duration,
    pub peek_timeout: Duration,
    pub pause_timeout: Duration,
}

impl Default for ProducerConsumerOptions {
    fn default() -> Self {
        ProducerConsumerOptions {
            capacity: CAPACITY_DEF,
            threshold: THRESHOLD_DEF,
            sleep_after_send: SLEEP_AFTER_SEND_DEF,
            peek_timeout: PEEK_TIMEOUT_DEF.clamp(PEEK_TIMEOUT_MIN, PEEK_TIMEOUT_MAX),
            pause_timeout: PAUSE_TIMEOUT_DEF.clamp(PAUSE_TIMEOUT_MIN, PAUSE_TIMEOUT_MAX),
        }
    }
}

impl ProducerConsumerOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(&self, capacity: usize) -> Self {
        ProducerConsumerOptions {
            capacity,
            ..self.clone()
        }
    }

    pub fn with_threshold(&self, threshold: Duration) -> Self {
        ProducerConsumerOptions {
            threshold,
            ..self.clone()
        }
    }

    pub fn with_sleep_after_send(&self, sleep_after_send: Duration) -> Self {
        ProducerConsumerOptions {
            sleep_after_send,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Producer<T: Send + Clone + 'static> {
    pc: Arc<ProducerConsumer<T>>,
    sender: Arc<channel::Sender<T>>,
}

impl<T: Send + Clone> Producer<T> {
    fn new(pc: &ProducerConsumer<T>, sender: &channel::Sender<T>) -> Self {
        Producer {
            pc: Arc::new(pc.clone()),
            sender: Arc::new(sender.clone()),
        }
    }

    pub fn enqueue(&self, item: T) {
        if self.pc.is_cancelled() {
            panic!("Queue is already cancelled.")
        }

        if self.pc.is_completed() {
            panic!("Queue is already completed.")
        }

        self.sender.send(item).unwrap();

        if !self.pc.options.sleep_after_send.is_zero() {
            thread::sleep(self.pc.options.sleep_after_send);
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProducerConsumer<T: Send + Clone + 'static> {
    options: ProducerConsumerOptions,
    finished: Arc<Mutex<bool>>,
    finished_cond: Arc<Condvar>,
    finished_notify: Arc<Notify>,
    completed: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    consumers_count: Arc<AtomicUsize>,
    running_count: Arc<AtomicUsize>,
    sender: channel::Sender<T>,
    receiver: channel::Receiver<T>,
}

impl<T: Send + Clone> ProducerConsumer<T> {
    pub fn new() -> Self {
        let options: ProducerConsumerOptions = Default::default();
        let (sender, receiver) = channel::bounded::<T>(options.capacity);
        ProducerConsumer {
            options,
            sender,
            receiver,
            finished: Arc::new(Mutex::new(false)),
            finished_cond: Arc::new(Condvar::new()),
            finished_notify: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
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
            options,
            sender,
            receiver,
            finished: Arc::new(Mutex::new(false)),
            finished_cond: Arc::new(Condvar::new()),
            finished_notify: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            consumers_count: Arc::new(AtomicUsize::new(0)),
            running_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn is_completed(&self) -> bool {
        self.completed.load(Ordering::SeqCst)
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn is_busy(&self) -> bool {
        self.running_count.load(Ordering::SeqCst) > 0
    }

    pub fn consumers(&self) -> usize {
        self.consumers_count.load(Ordering::SeqCst)
    }

    fn inc_consumers(&self) {
        self.consumers_count.fetch_add(1, Ordering::SeqCst);
    }

    fn dec_consumers(&self, td: &dyn ProducerConsumerDelegation<T>) {
        self.consumers_count.fetch_sub(1, Ordering::SeqCst);
        self.check_finished(td);
    }

    fn check_finished(&self, td: &dyn ProducerConsumerDelegation<T>) {
        if self.is_completed() && self.consumers() == 0 {
            let mut finished = self.finished.lock().unwrap();
            *finished = true;
            td.on_finished(self);
            self.finished_cond.notify_all();
            self.finished_notify.notify_waiters();
        }
    }

    pub fn running(&self) -> usize {
        self.running_count.load(Ordering::SeqCst)
    }

    fn inc_running(&self) {
        self.running_count.fetch_add(1, Ordering::SeqCst);
    }

    fn dec_running(&self) {
        self.running_count.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn new_producer(&self) -> Producer<T> {
        if self.is_cancelled() {
            panic!("Queue is already cancelled.")
        }

        if self.is_completed() {
            panic!("Queue is already completed.")
        }

        Producer::new(self, &self.sender)
    }

    pub fn start_consumer<S: ProducerConsumerDelegation<T> + Send + Clone + 'static>(
        &self,
        delegate: S,
    ) {
        if self.is_cancelled() {
            panic!("Queue is already cancelled.")
        }

        self.inc_consumers();
        let this = Arc::new(self.clone());
        let builder = thread::Builder::new().name(format!("Consumer {}", self.consumers()));

        if self.options.threshold.is_zero() {
            builder.spawn(move || this.run_consumer(&delegate)).unwrap();
        } else {
            builder
                .spawn(move || this.run_consumer_with_threshold(&delegate))
                .unwrap();
        }
    }

    fn run_consumer<S: ProducerConsumerDelegation<T> + Send + Clone + 'static>(
        &self,
        delegate: &S,
    ) {
        let delegate = delegate.clone();

        loop {
            if self.is_cancelled() || (self.is_completed() && self.running() == 0) {
                break;
            }

            let Ok(item) = self.receiver.recv_timeout(self.options.peek_timeout) else {
                continue;
            };
            self.inc_running();

            if let Ok(result) = delegate.process(self, &item) {
                if !delegate.on_completed(self, &item, result) {
                    self.dec_running();
                    break;
                }
            }

            self.dec_running();
        }

        self.dec_consumers(&delegate);
        drop(delegate);
    }

    fn run_consumer_with_threshold<S: ProducerConsumerDelegation<T> + Send + Clone + 'static>(
        &self,
        delegate: &S,
    ) {
        let delegate = delegate.clone();

        loop {
            if self.is_cancelled() || (self.is_completed() && self.running() == 0) {
                break;
            }

            let Ok(item) = self.receiver.recv_timeout(self.options.peek_timeout) else {
                continue;
            };
            self.inc_running();

            if let Ok(result) = delegate.process(&self, &item) {
                let time = Instant::now();

                if !delegate.on_completed(&self, &item, result) {
                    self.dec_running();
                    break;
                }

                if !self.options.threshold.is_zero() && time.elapsed() < self.options.threshold {
                    let remaining = self.options.threshold - time.elapsed();
                    thread::sleep(remaining);
                }
            }

            self.dec_running();
        }

        self.dec_consumers(&delegate);
        drop(delegate);
    }

    pub fn stop(&self, enforce: bool) {
        if enforce {
            self.cancel();
        } else {
            self.complete();
        }
    }

    pub fn complete(&self) {
        self.completed.store(true, Ordering::SeqCst);
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn wait(&self) {
        let finished = self.finished.lock().unwrap();

        if !*finished {
            let _ignored = self.finished_cond.wait(finished).unwrap();
        }
    }

    pub async fn wait_async(&self) {
        self.finished_notify.notified().await;
    }

    pub fn wait_for(&self, timeout: Duration) -> bool {
        if timeout.is_zero() {
            self.wait();
            return true;
        }

        let start = Instant::now();
        let mut finished = self.finished.lock().unwrap();

        while !*finished && start.elapsed() < timeout {
            let result = self
                .finished_cond
                .wait_timeout(finished, self.options.pause_timeout)
                .unwrap();
            finished = result.0;
            thread::sleep(self.options.pause_timeout);

            if result.1.timed_out() || start.elapsed() >= timeout {
                return false;
            }
        }

        start.elapsed() < timeout
    }

    pub async fn wait_for_async(&self, timeout: Duration) -> bool {
        if timeout.is_zero() {
            self.wait_async().await;
            return true;
        }

        let start = Instant::now();
        let finished = self.finished.lock().unwrap();

        while !*finished && start.elapsed() < timeout {
            let result =
                tokio::time::timeout(self.options.pause_timeout, self.finished_notify.notified())
                    .await
                    .is_ok();

            if !result {
                return false;
            }

            thread::sleep(self.options.pause_timeout);
        }

        start.elapsed() < timeout
    }
}
