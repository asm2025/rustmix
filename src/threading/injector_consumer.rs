use anyhow::Result;
use crossbeam::deque::{Injector, Steal, Stealer, Worker};
use std::{
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

pub trait InjectorWorkerDelegation<T: Send + Clone + 'static> {
    fn process(&self, pc: &InjectorWorker<T>, item: &T) -> Result<TaskResult>;
    fn on_started(&self, pc: &InjectorWorker<T>);
    fn on_completed(&self, pc: &InjectorWorker<T>, item: &T, result: TaskResult) -> bool;
    fn on_finished(&self, pc: &InjectorWorker<T>);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectorWorkerOptions {
    pub threads: usize,
    pub threshold: Duration,
    pub sleep_after_send: Duration,
    pub pause_timeout: Duration,
}

impl Default for InjectorWorkerOptions {
    fn default() -> Self {
        InjectorWorkerOptions {
            threads: THREADS_DEF.clamp(THREADS_MIN, THREADS_MAX),
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

    pub fn with_threads(&self, threads: usize) -> Self {
        InjectorWorkerOptions {
            threads: threads.clamp(THREADS_MIN, THREADS_MAX),
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
    finishedc: Arc<Condvar>,
    finishedn: Arc<Notify>,
    completed: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    workers: Arc<AtomicUsize>,
    running: Arc<AtomicUsize>,
}

impl<T: Send + Clone> InjectorWorker<T> {
    pub fn new() -> Self {
        let options: InjectorWorkerOptions = Default::default();
        InjectorWorker {
            options,
            injector: Arc::new(Injector::new()),
            stealers: Arc::new(Mutex::new(Vec::new())),
            started: Arc::new(Mutex::new(false)),
            finished: Arc::new(Mutex::new(false)),
            finishedc: Arc::new(Condvar::new()),
            finishedn: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            workers: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn with_options(options: InjectorWorkerOptions) -> Self {
        InjectorWorker {
            options,
            injector: Arc::new(Injector::new()),
            stealers: Arc::new(Mutex::new(Vec::new())),
            started: Arc::new(Mutex::new(false)),
            finished: Arc::new(Mutex::new(false)),
            finishedc: Arc::new(Condvar::new()),
            finishedn: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            workers: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicUsize::new(0)),
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
        self.completed.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn is_busy(&self) -> bool {
        (self.running.load(Ordering::SeqCst) > 0)
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
        self.workers.load(Ordering::SeqCst)
    }

    fn inc_workers(&self) {
        self.workers.fetch_add(1, Ordering::SeqCst);
    }

    fn dec_workers(&self, td: &dyn InjectorWorkerDelegation<T>) {
        self.workers.fetch_sub(1, Ordering::SeqCst);
        self.check_finished(td);
    }

    fn check_finished(&self, td: &dyn InjectorWorkerDelegation<T>) {
        if self.is_completed() && self.workers() == 0 {
            let mut finished = self.finished.lock().unwrap();
            *finished = true;
            td.on_finished(self);
            self.set_started(false);
            self.finishedc.notify_all();
            self.finishedn.notify_waiters();
        }
    }

    pub fn running(&self) -> usize {
        self.running.load(Ordering::SeqCst)
    }

    fn inc_running(&self) {
        self.running.fetch_add(1, Ordering::SeqCst);
    }

    fn dec_running(&self) {
        self.running.fetch_sub(1, Ordering::SeqCst);
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

        delegate.on_started(self);
        let mut stealers = self.stealers.lock().unwrap();

        for _ in 0..self.options.threads {
            let worker = Worker::<T>::new_fifo();
            let stealer = worker.stealer();
            stealers.push(stealer);
            self.inc_workers();
            let this = self.clone();
            let del = delegate.clone();
            let builder = thread::Builder::new().name(format!("Worker {}", self.workers()));

            if self.options.threshold.is_zero() {
                builder
                    .spawn(move || this.run_consumer(worker, del))
                    .unwrap();
            } else {
                builder
                    .spawn(move || this.run_consumer_with_threshold(worker, del))
                    .unwrap();
            }
        }
    }

    fn run_consumer<S: InjectorWorkerDelegation<T> + Send + Clone + 'static>(
        &self,
        worker: Worker<T>,
        delegate: S,
    ) {
        let this = Arc::new(self.clone());
        let global = this.injector.clone();
        let local = Arc::new(Mutex::new(worker));

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

                    if let Ok(result) = delegate.process(&this, &item) {
                        if !delegate.on_completed(&this, &item, result) {
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
        drop(this);
        drop(global);
        drop(local);
    }

    fn run_consumer_with_threshold<S: InjectorWorkerDelegation<T> + Send + Clone + 'static>(
        &self,
        worker: Worker<T>,
        delegate: S,
    ) {
        let this = Arc::new(self.clone());
        let global = this.injector.clone();
        let local = Arc::new(Mutex::new(worker));

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

                    if let Ok(result) = delegate.process(&this, &item) {
                        let time = Instant::now();

                        if !delegate.on_completed(&this, &item, result) {
                            this.dec_running();
                            break;
                        }

                        if !this.options.threshold.is_zero()
                            && time.elapsed() < this.options.threshold
                        {
                            let remaining = this.options.threshold - time.elapsed();
                            thread::sleep(remaining);
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
        drop(this);
        drop(global);
        drop(local);
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
        self.completed.store(true, Ordering::SeqCst);
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
    }

    pub fn wait(&self) {
        let finished = self.finished.lock().unwrap();

        if !*finished {
            let _ignored = self.finishedc.wait(finished).unwrap();
        }
    }

    pub async fn wait_async(&self) {
        while !*self.finished.lock().unwrap() {
            self.finishedn.notified().await;
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
                .finishedc
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
                .finishedc
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
