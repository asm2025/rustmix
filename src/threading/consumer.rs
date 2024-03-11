use std::{
    collections::LinkedList,
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

pub trait ConsumerDelegation<T: Send + Clone + 'static> {
    fn process(&self, pc: &Consumer<T>, item: &T) -> Result<TaskResult, Box<dyn Error>>;
    fn on_completed(&self, pc: &Consumer<T>, item: &T, result: TaskResult) -> bool;
    fn on_finished(&self, pc: &Consumer<T>);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerOptions {
    pub behavior: QueueBehavior,
    pub threshold: Duration,
    pub sleep_after_send: Duration,
    pub peek_timeout: Duration,
    pub pause_timeout: Duration,
}

impl Default for ConsumerOptions {
    fn default() -> Self {
        ConsumerOptions {
            behavior: QUEUE_BEHAVIOR_DEF,
            threshold: THRESHOLD_DEF,
            sleep_after_send: SLEEP_AFTER_SEND_DEF,
            peek_timeout: PEEK_TIMEOUT_DEF.clamp(PEEK_TIMEOUT_MIN, PEEK_TIMEOUT_MAX),
            pause_timeout: PAUSE_TIMEOUT_DEF.clamp(PAUSE_TIMEOUT_MIN, PAUSE_TIMEOUT_MAX),
        }
    }
}

impl ConsumerOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_behavior(&self, behavior: QueueBehavior) -> Self {
        ConsumerOptions {
            behavior,
            ..self.clone()
        }
    }

    pub fn with_threshold(&self, threshold: Duration) -> Self {
        ConsumerOptions {
            threshold,
            ..self.clone()
        }
    }

    pub fn with_sleep_after_send(&self, sleep_after_send: Duration) -> Self {
        ConsumerOptions {
            sleep_after_send,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Consumer<T: Send + Clone + 'static> {
    options: ConsumerOptions,
    items: Arc<Mutex<LinkedList<T>>>,
    items_cond: Arc<Condvar>,
    finished: Arc<Mutex<bool>>,
    finished_cond: Arc<Condvar>,
    finished_notify: Arc<Notify>,
    completed: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    consumers_count: Arc<AtomicUsize>,
    running_count: Arc<AtomicUsize>,
}

impl<T: Send + Clone> Consumer<T> {
    pub fn new() -> Self {
        let options: ConsumerOptions = Default::default();
        Consumer {
            options: options,
            items: Arc::new(Mutex::new(LinkedList::new())),
            items_cond: Arc::new(Condvar::new()),
            finished: Arc::new(Mutex::new(false)),
            finished_cond: Arc::new(Condvar::new()),
            finished_notify: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            consumers_count: Arc::new(AtomicUsize::new(0)),
            running_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn with_options(options: ConsumerOptions) -> Self {
        Consumer {
            options: options,
            items: Arc::new(Mutex::new(LinkedList::new())),
            items_cond: Arc::new(Condvar::new()),
            finished: Arc::new(Mutex::new(false)),
            finished_cond: Arc::new(Condvar::new()),
            finished_notify: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            consumers_count: Arc::new(AtomicUsize::new(0)),
            running_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.lock().unwrap().is_empty()
    }

    pub fn is_completed(&self) -> bool {
        self.completed.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn is_busy(&self) -> bool {
        (self.consumers_count.load(Ordering::SeqCst) > 0)
            || (self.running_count.load(Ordering::SeqCst) > 0)
            || (self.items.lock().unwrap().len() > 0)
    }

    pub fn count(&self) -> usize {
        self.items.lock().unwrap().len()
    }

    pub fn consumers(&self) -> usize {
        self.consumers_count.load(Ordering::SeqCst)
    }

    fn inc_consumers(&self) {
        self.consumers_count.fetch_add(1, Ordering::SeqCst);
    }

    fn dec_consumers(&self, td: &dyn ConsumerDelegation<T>) {
        self.consumers_count.fetch_sub(1, Ordering::SeqCst);
        self.check_finished(td);
    }

    fn check_finished(&self, td: &dyn ConsumerDelegation<T>) {
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

    pub fn start<S: ConsumerDelegation<T> + Send + Clone + 'static>(&self, delegate: S) {
        if self.is_cancelled() {
            panic!("Queue is already cancelled.")
        }

        if self.is_completed() && self.is_empty() {
            panic!("Queue is already completed.")
        }

        self.inc_consumers();
        let builder = thread::Builder::new().name(format!("Consumer {}", self.consumers()));
        let this = Arc::new(self.clone());

        if self.options.threshold.is_zero() {
            builder
                .spawn(move || {
                    this.run_consumer(&delegate);
                })
                .unwrap();
        } else {
            builder
                .spawn(move || {
                    this.run_consumer_with_threshold(&delegate);
                })
                .unwrap();
        }
    }

    fn run_consumer<S: ConsumerDelegation<T> + Send + Clone + 'static>(&self, delegate: &S) {
        let delegate = delegate.clone();

        loop {
            if self.is_cancelled()
                || (self.is_empty() && self.is_completed() && self.running() == 0)
            {
                break;
            }

            if self.is_paused() {
                thread::sleep(self.options.pause_timeout);
                continue;
            }

            if let Some(item) = match self.options.behavior {
                QueueBehavior::FIFO => self.dequeue(),
                QueueBehavior::LIFO => self.pop(),
            } {
                self.inc_running();

                if let Ok(result) = delegate.process(self, &item) {
                    if !delegate.on_completed(self, &item, result) {
                        self.dec_running();
                        break;
                    }
                }

                self.dec_running();
            }
        }

        self.dec_consumers(&delegate);
        drop(delegate);
    }

    fn run_consumer_with_threshold<S: ConsumerDelegation<T> + Send + Clone + 'static>(
        &self,
        delegate: &S,
    ) {
        let delegate = delegate.clone();

        loop {
            if self.is_cancelled()
                || (self.is_empty() && self.is_completed() && self.running() == 0)
            {
                break;
            }

            if self.is_paused() {
                thread::sleep(self.options.pause_timeout);
                continue;
            }

            if let Some(item) = match self.options.behavior {
                QueueBehavior::FIFO => self.dequeue(),
                QueueBehavior::LIFO => self.pop(),
            } {
                self.inc_running();

                if let Ok(result) = delegate.process(&self, &item) {
                    let time = Instant::now();

                    if !delegate.on_completed(&self, &item, result) {
                        self.dec_running();
                        break;
                    }

                    if !self.options.threshold.is_zero() && time.elapsed() < self.options.threshold
                    {
                        let remaining = self.options.threshold - time.elapsed();
                        thread::sleep(remaining);
                    }
                }

                self.dec_running();
            }
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

    pub fn enqueue(&self, item: T) {
        if self.is_cancelled() {
            panic!("Queue is already cancelled.")
        }

        if self.is_completed() {
            panic!("Queue is already completed.")
        }

        let mut items = self.items.lock().unwrap();
        items.push_back(item);

        if !self.options.sleep_after_send.is_zero() {
            thread::sleep(self.options.sleep_after_send);
        }

        self.items_cond.notify_one();
    }

    fn dequeue(&self) -> Option<T> {
        let mut items = self.items.lock().unwrap();

        while items.is_empty() && !self.is_cancelled() && !self.is_completed() {
            let result = self
                .items_cond
                .wait_timeout(items, self.options.peek_timeout)
                .unwrap();
            items = result.0;

            if result.1.timed_out() {
                continue;
            }

            if self.is_cancelled() || self.is_completed() {
                return None;
            }

            return items.pop_front();
        }

        if items.is_empty() || self.is_cancelled() {
            return None;
        }

        items.pop_front()
    }

    fn pop(&self) -> Option<T> {
        let mut items = self.items.lock().unwrap();

        while items.is_empty() && !self.is_cancelled() && !self.is_completed() {
            let result = self
                .items_cond
                .wait_timeout(items, self.options.peek_timeout)
                .unwrap();
            items = result.0;

            if result.1.timed_out() {
                continue;
            }

            if self.is_cancelled() || self.is_completed() {
                return None;
            }

            return items.pop_back();
        }

        if items.is_empty() || self.is_cancelled() {
            return None;
        }

        items.pop_back()
    }

    pub fn peek(&self) -> Option<T> {
        let items = self.items.lock().unwrap();

        if items.is_empty() {
            return None;
        }

        if let Some(item) = match self.options.behavior {
            QueueBehavior::FIFO => items.front(),
            QueueBehavior::LIFO => items.back(),
        } {
            Some(item.clone())
        } else {
            None
        }
    }

    pub fn clear(&self) {
        let mut items = self.items.lock().unwrap();
        items.clear();
    }

    pub fn complete(&self) {
        self.completed.store(true, Ordering::SeqCst);
        self.items_cond.notify_all();
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.items_cond.notify_all();
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
        self.items_cond.notify_all();
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
            thread::sleep(self.options.pause_timeout);
        }
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

    pub async fn wait_for_async(&self, timeout: Duration) -> Box<dyn Future<Output = bool>> {
        if timeout.is_zero() {
            self.wait_async().await;
            return Box::new(async { true });
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
                return Box::new(async move { false });
            }
        }

        Box::new(async move { start.elapsed() < timeout })
    }
}
