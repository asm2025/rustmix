use rayon::{prelude::*, ThreadPoolBuilder};
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

pub trait ParallelDelegation<T: Send + 'static> {
    fn process(&self, pc: &Parallel, item: &T) -> Result<TaskResult, Box<dyn Error>>;
    fn on_completed(&self, pc: &Parallel, item: &T, result: TaskResult) -> bool;
    fn on_finished(&self, pc: &Parallel);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParallelOptions {
    pub threads: usize,
    pub threshold: Duration,
    pub pause_timeout: Duration,
}

impl Default for ParallelOptions {
    fn default() -> Self {
        ParallelOptions {
            threads: THREADS_DEF.clamp(THREADS_MIN, THREADS_MAX),
            threshold: THRESHOLD_DEF,
            pause_timeout: PAUSE_TIMEOUT_DEF.clamp(PAUSE_TIMEOUT_MIN, PAUSE_TIMEOUT_MAX),
        }
    }
}

impl ParallelOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_threads(&self, threads: usize) -> Self {
        ParallelOptions {
            threads: threads.clamp(THREADS_MIN, THREADS_MAX),
            ..self.clone()
        }
    }

    pub fn with_threshold(&self, threshold: Duration) -> Self {
        ParallelOptions {
            threshold,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Parallel {
    options: ParallelOptions,
    finished: Arc<Mutex<bool>>,
    finishedc: Arc<Condvar>,
    finishedn: Arc<Notify>,
    completed: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    running: Arc<AtomicUsize>,
}

impl Parallel {
    pub fn new() -> Self {
        let options: ParallelOptions = Default::default();
        Parallel {
            options,
            finished: Arc::new(Mutex::new(false)),
            finishedc: Arc::new(Condvar::new()),
            finishedn: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn with_options(options: ParallelOptions) -> Self {
        Parallel {
            options,
            finished: Arc::new(Mutex::new(false)),
            finishedc: Arc::new(Condvar::new()),
            finishedn: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicUsize::new(0)),
        }
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
        self.running.load(Ordering::SeqCst) > 0
    }

    pub fn running(&self) -> usize {
        self.running.load(Ordering::SeqCst)
    }

    fn inc_running(&self) {
        self.running.fetch_add(1, Ordering::SeqCst);
    }

    fn dec_running<T: Send + 'static>(&self, td: &dyn ParallelDelegation<T>) {
        self.running.fetch_sub(1, Ordering::SeqCst);
        self.check_finished(td);
    }

    fn check_finished<T: Send + 'static>(&self, td: &dyn ParallelDelegation<T>) {
        if self.running() == 0 {
            let mut finished = self.finished.lock().unwrap();
            *finished = true;
            td.on_finished(self);
            self.finishedc.notify_all();
            self.finishedn.notify_waiters();
        }
    }

    pub fn start<
        T: Send + 'static,
        I: IntoParallelIterator<Item = T> + Send,
        S: ParallelDelegation<T> + Send + Clone + Sync + 'static,
    >(
        &self,
        collection: I,
        delegate: S,
    ) {
        if self.is_cancelled() {
            panic!("Queue is already cancelled.")
        }

        if self.options.threshold.is_zero() {
            self.run_consumer(collection, delegate);
        } else {
            self.run_consumer_with_threshold(collection, delegate);
        }
    }

    fn run_consumer<
        T: Send + 'static,
        I: IntoParallelIterator<Item = T> + Send,
        S: ParallelDelegation<T> + Send + Clone + Sync + 'static,
    >(
        &self,
        collection: I,
        delegate: S,
    ) {
        let pool = ThreadPoolBuilder::new()
            .num_threads(self.options.threads)
            .build()
            .unwrap();
        pool.install(move || {
            collection.into_par_iter().for_each(|item| {
                while !self.is_cancelled() && self.is_paused() {
                    thread::sleep(self.options.pause_timeout);
                }

                if self.is_cancelled() {
                    return;
                }

                self.inc_running();

                if let Ok(result) = delegate.process(self, &item) {
                    if !delegate.on_completed(self, &item, result) {
                        self.cancel();
                        return;
                    }
                }

                self.dec_running(&delegate);
            });
        });
    }

    fn run_consumer_with_threshold<
        T: Send + 'static,
        I: IntoParallelIterator<Item = T> + Send,
        S: ParallelDelegation<T> + Send + Clone + Sync + 'static,
    >(
        &self,
        collection: I,
        delegate: S,
    ) {
        let pool = ThreadPoolBuilder::new()
            .num_threads(self.options.threads)
            .build()
            .unwrap();
        pool.install(move || {
            collection.into_par_iter().for_each(|item| {
                while !self.is_cancelled() && self.is_paused() {
                    thread::sleep(self.options.pause_timeout);
                }

                if self.is_cancelled() {
                    return;
                }

                self.inc_running();

                if let Ok(result) = delegate.process(self, &item) {
                    let time = Instant::now();

                    if !delegate.on_completed(self, &item, result) {
                        self.cancel();
                        return;
                    }

                    if !self.options.threshold.is_zero() && time.elapsed() < self.options.threshold
                    {
                        let remaining = self.options.threshold - time.elapsed();
                        thread::sleep(remaining);
                    }
                }

                self.dec_running(&delegate);
            });
        });
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
