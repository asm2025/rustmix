use rayon::{prelude::*, ThreadPoolBuilder};
use std::{
    iter::ExactSizeIterator,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
};
use tokio::{sync::Notify, time::Duration};

use super::{cond::Mutcond, *};
use crate::{error::*, Result};

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
pub struct Parallel<T: StaticTaskItem> {
    pub options: ParallelOptions,
    started: Arc<Mutex<bool>>,
    finished: Arc<AtomicBool>,
    finished_cond: Arc<Mutcond>,
    finished_noti: Arc<Notify>,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    running: Arc<AtomicUsize>,
    _marker: PhantomData<T>,
}

impl<T: StaticTaskItem> Parallel<T> {
    pub fn new() -> Self {
        Parallel {
            options: Default::default(),
            started: Arc::new(Mutex::new(false)),
            finished: Arc::new(AtomicBool::new(false)),
            finished_cond: Arc::new(Mutcond::new()),
            finished_noti: Arc::new(Notify::new()),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicUsize::new(0)),
            _marker: PhantomData,
        }
    }

    pub fn with_options(options: ParallelOptions) -> Self {
        Parallel {
            options,
            started: Arc::new(Mutex::new(false)),
            finished: Arc::new(AtomicBool::new(false)),
            finished_cond: Arc::new(Mutcond::new()),
            finished_noti: Arc::new(Notify::new()),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicUsize::new(0)),
            _marker: PhantomData,
        }
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

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::SeqCst)
    }

    pub fn is_busy(&self) -> bool {
        self.len() > 0
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.running.load(Ordering::SeqCst)
    }

    pub fn running(&self) -> usize {
        self.running.load(Ordering::SeqCst)
    }

    fn set_running(&self, value: usize) {
        self.running.store(value, Ordering::SeqCst);
    }

    fn dec_running(&self) -> bool {
        let prev = self.running.fetch_sub(1, Ordering::SeqCst);
        prev == 1 || self.is_cancelled()
    }

    fn finish(&self) {
        if self.running() > 0 && !self.is_cancelled() {
            return;
        }

        self.finished.store(true, Ordering::SeqCst);
        self.set_started(false);
        self.finished_cond.notify_all();
        self.finished_noti.notify_waiters();
        thread::sleep(Duration::ZERO);
    }

    pub fn start<
        I: ExactSizeIterator<Item = T> + ParallelIterator<Item = T> + Send + 'static,
        H: TaskDelegation<Parallel<T>, T>,
    >(
        &self,
        collection: I,
        handler: &H,
    ) -> Result<()> {
        if self.is_cancelled() {
            return Err(CancelledError.into());
        }

        if !self.set_started(true) {
            return Err(QueueStartedError.into());
        }

        self.set_running(collection.len());
        handler.on_started();

        let pool = ThreadPoolBuilder::new()
            .num_threads(self.options.threads)
            .build()
            .unwrap();
        let this = self.clone();
        let handler = handler.clone();
        thread::spawn(move || {
            if this.options.threshold.is_zero() {
                pool.install(move || {
                    ParallelIterator::for_each(collection.into_par_iter(), |item| {
                        while !this.is_cancelled() && this.is_paused() {
                            thread::sleep(this.options.pause_timeout);
                        }

                        if this.is_cancelled() {
                            this.dec_running();
                            return;
                        }

                        match handler.process(&this, &item) {
                            Ok(result) => {
                                if !handler.on_completed(&item, &result) {
                                    this.cancel();
                                    return;
                                }
                            }
                            Err(e) => {
                                if !handler.on_completed(&item, &TaskResult::Error(e.get_message()))
                                {
                                    this.cancel();
                                    return;
                                }
                            }
                        }

                        if !this.dec_running() {
                            return;
                        }

                        if this.is_cancelled() {
                            handler.on_cancelled();
                        } else {
                            handler.on_finished();
                        }

                        this.finish();
                    });
                });

                return;
            }

            pool.install(move || {
                ParallelIterator::for_each(collection.into_par_iter(), |item| {
                    while !this.is_cancelled() && this.is_paused() {
                        thread::sleep(this.options.pause_timeout);
                    }

                    if this.is_cancelled() {
                        this.dec_running();
                        return;
                    }

                    match handler.process(&this, &item) {
                        Ok(result) => {
                            if !handler.on_completed(&item, &result) {
                                this.cancel();
                                return;
                            }
                        }
                        Err(e) => {
                            if !handler.on_completed(&item, &TaskResult::Error(e.get_message())) {
                                this.cancel();
                                return;
                            }
                        }
                    }

                    if !this.dec_running() {
                        return;
                    }

                    if this.is_cancelled() {
                        handler.on_cancelled();
                    } else {
                        handler.on_finished();
                    }

                    this.finish();
                });
            });
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.cancel();
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

    pub fn wait(&self) -> Result<()> {
        wait(self, &self.finished_cond)
    }

    pub async fn wait_async(&self) -> Result<()> {
        wait_async(self, &self.finished_noti).await
    }

    pub fn wait_until(&self, cond: impl Fn(&Parallel<T>) -> bool) -> Result<()> {
        wait_until(self, &self.finished_cond, cond)
    }

    pub async fn wait_until_async(
        &self,
        cond: impl Fn(&Parallel<T>) -> Pin<Box<dyn Future<Output = bool> + Send>>,
    ) -> Result<()> {
        wait_until_async(self, &self.finished_noti, cond).await
    }

    pub fn wait_for(&self, timeout: Duration) -> Result<()> {
        wait_for(self, timeout, &self.finished_cond)
    }

    pub async fn wait_for_async(&self, timeout: Duration) -> Result<()> {
        wait_for_async(self, timeout, &self.finished_noti).await
    }

    pub fn wait_for_until(
        &self,
        timeout: Duration,
        cond: impl Fn(&Parallel<T>) -> bool,
    ) -> Result<()> {
        wait_for_until(self, timeout, &self.finished_cond, cond)
    }

    pub async fn wait_for_until_async<
        F: Fn(&Parallel<T>) -> Pin<Box<dyn Future<Output = bool> + Send>>,
    >(
        &self,
        timeout: Duration,
        cond: F,
    ) -> Result<()> {
        wait_for_until_async(self, timeout, &self.finished_noti, cond).await
    }
}

impl<T: StaticTaskItem> AwaitableConsumer<T> for Parallel<T> {
    fn is_cancelled(&self) -> bool {
        Parallel::is_cancelled(self)
    }

    fn is_finished(&self) -> bool {
        Parallel::is_finished(self)
    }
}
