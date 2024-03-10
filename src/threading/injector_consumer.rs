use crossbeam::deque::{Injector, Steal, Stealer, Worker};
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

use super::*;

pub trait InjectorWorkerDelegation<T: Send + Clone + 'static> {
    fn process_task(&self, pc: &InjectorWorker<T>, item: T) -> Result<TaskResult, Box<dyn Error>>;
    fn on_task_completed(&self, pc: &InjectorWorker<T>, item: T, result: TaskResult) -> bool;
    fn on_finished(&self, pc: &InjectorWorker<T>);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectorWorkerOptions {
    pub threads: usize,
    pub capacity: usize,
    pub threshold: Duration,
    pub sleep_after_send: Duration,
    pub pause_timeout: Duration,
}

impl Default for InjectorWorkerOptions {
    fn default() -> Self {
        InjectorWorkerOptions {
            capacity: CAPACITY_DEF,
            threads: THREADS_DEF,
            threshold: THRESHOLD_DEF,
            sleep_after_send: SLEEP_AFTER_SEND_DEF,
            pause_timeout: PAUSE_TIMEOUT_DEF.clamp(PAUSE_TIMEOUT_MIN, PAUSE_TIMEOUT_MAX),
        }
    }
}

impl InjectorWorkerOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(&self, capacity: usize) -> Self {
        InjectorWorkerOptions {
            capacity,
            ..self.clone()
        }
    }

    pub fn with_threads(&self, threads: usize) -> Self {
        InjectorWorkerOptions {
            threads: if threads > 0 { threads } else { 1 },
            ..self.clone()
        }
    }

    pub fn with_threshold(&self, threshold: Duration) -> Self {
        InjectorWorkerOptions {
            threshold,
            ..self.clone()
        }
    }

    pub fn with_sleep_after_send(&self, sleep_after_send: Duration) -> Self {
        InjectorWorkerOptions {
            sleep_after_send,
            ..self.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct InjectorWorker<T: Send + Clone + 'static> {
    options: InjectorWorkerOptions,
    injector: Arc<Injector<T>>,
    stealers: Arc<Mutex<Vec<Stealer<T>>>>,
    started: Arc<Mutex<bool>>,
    finished: Arc<Mutex<bool>>,
    finished_cond: Arc<Condvar>,
    finished_notify: Arc<Notify>,
    completed: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    workers_count: Arc<AtomicUsize>,
    running_count: Arc<AtomicUsize>,
}

impl<T: Send + Clone> InjectorWorker<T> {
    pub fn new() -> Self {
        let options: InjectorWorkerOptions = Default::default();
        InjectorWorker {
            options: options,
            injector: Arc::new(Injector::new()),
            stealers: Arc::new(Mutex::new(Vec::new())),
            started: Arc::new(Mutex::new(false)),
            finished: Arc::new(Mutex::new(false)),
            finished_cond: Arc::new(Condvar::new()),
            finished_notify: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            workers_count: Arc::new(AtomicUsize::new(0)),
            running_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn with_options(options: InjectorWorkerOptions) -> Self {
        InjectorWorker {
            options: options,
            injector: Arc::new(Injector::new()),
            stealers: Arc::new(Mutex::new(Vec::new())),
            started: Arc::new(Mutex::new(false)),
            finished: Arc::new(Mutex::new(false)),
            finished_cond: Arc::new(Condvar::new()),
            finished_notify: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            workers_count: Arc::new(AtomicUsize::new(0)),
            running_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.injector.is_empty()
    }

    pub fn is_started(&self) -> bool {
        *self.started.lock().unwrap()
    }

    fn set_started(&self, value: bool) -> bool {
        let mut started = self.started.lock().unwrap();

        if *started && value {
            return false;
        }

        *started = true;
        true
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
            .running_count
            .load(std::sync::atomic::Ordering::Relaxed)
            > 0)
            || (self.injector.len() > 0)
            || (self.workers() > 0)
    }

    pub fn count(&self) -> usize {
        self.injector.len()
            + self
                .stealers
                .lock()
                .unwrap()
                .iter()
                .map(|s| s.len())
                .sum::<usize>()
    }

    pub fn workers(&self) -> usize {
        self.workers_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn inc_workers(&self) {
        self.workers_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn dec_workers(&self, td: &dyn InjectorWorkerDelegation<T>) {
        self.workers_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        self.check_finished(td);
    }

    fn check_finished(&self, td: &dyn InjectorWorkerDelegation<T>) {
        if self.is_completed() && self.workers() == 0 {
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

    pub fn start<S: InjectorWorkerDelegation<T> + Send + Clone + 'static>(&self, delegate: S) {
        if self.is_cancelled() {
            panic!("Queue is already cancelled.")
        }

        if self.is_completed() && self.is_empty() {
            panic!("Queue is already completed.")
        }

        if !self.set_started(true) {
            return;
        }

        let mut stealers = self.stealers.lock().unwrap();

        for _ in 0..self.options.threads {
            let worker = Worker::<T>::new_fifo();
            let stealer = worker.stealer();
            stealers.push(stealer);
            self.inc_workers();
            self.spawn(worker, delegate.clone());
        }
    }

    fn spawn<S: InjectorWorkerDelegation<T> + Send + Clone + 'static>(
        &self,
        worker: Worker<T>,
        delegate: S,
    ) {
        let this = Arc::new(self.clone());
        let global = this.injector.clone();
        let local = Arc::new(Mutex::new(worker));
        let delegate = delegate.clone();
        let builder = thread::Builder::new().name(format!("Worker {}", self.workers()));
        builder
            .spawn(move || {
                loop {
                    if this.is_cancelled() {
                        break;
                    }

                    if this.is_paused() {
                        thread::sleep(this.options.pause_timeout);
                        continue;
                    }

                    let local = local.lock().unwrap();
                    // Pop a task from the local queue, if not empty.
                    match local.pop().or_else(|| {
                        // Otherwise, we need to look for a task elsewhere.
                        if this.is_cancelled() {
                            return None;
                        }

                        if this.is_paused() {
                            thread::sleep(this.options.pause_timeout);
                            return None;
                        }

                        // Try stealing a batch of tasks from the global queue.
                        global
                            .steal_batch_with_limit_and_pop(&local, 10)
                            // Or try stealing a task from one of the other threads.
                            .or_else(|| {
                                this.stealers
                                    .lock()
                                    .unwrap()
                                    .iter()
                                    .map(|s| s.steal_batch_with_limit_and_pop(&local, 10))
                                    .find(|s| s.is_success())
                                    .unwrap_or_else(|| Steal::Empty)
                            })
                            .success()
                    }) {
                        Some(item) => {
                            this.inc_running();

                            if let Ok(result) = delegate.process_task(&this, item.clone()) {
                                if !delegate.on_task_completed(&this, item, result) {
                                    this.dec_running();
                                    break;
                                }
                            }

                            this.dec_running();
                        }
                        _ => {
                            if this.is_cancelled() || this.is_completed() {
                                break;
                            }
                        }
                    }
                }

                this.dec_workers(&delegate);
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
        if self.is_cancelled() {
            panic!("Queue is already cancelled.")
        }

        if self.is_completed() {
            panic!("Queue is already completed.")
        }

        self.injector.push(item);
    }

    pub fn complete(&self) {
        self.completed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn pause(&self) {
        self.paused
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.paused
            .store(false, std::sync::atomic::Ordering::Relaxed);
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
        if timeout == Duration::ZERO {
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
        if timeout == Duration::ZERO {
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
