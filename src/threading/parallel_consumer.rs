use futures::executor::block_on;
use rayon::{prelude::*, ThreadPoolBuilder};
use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tokio::{
    select,
    sync::Notify,
    time::{sleep as tokio_sleep, timeout as tokio_timeout},
};

use super::*;

pub trait Len {
    fn len(&self) -> usize;
}

impl<T, const N: usize> Len for [T; N] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> Len for BinaryHeap<T> {
    fn len(&self) -> usize {
        BinaryHeap::len(self)
    }
}

impl<K, V> Len for BTreeMap<K, V> {
    fn len(&self) -> usize {
        BTreeMap::len(self)
    }
}

impl<T> Len for BTreeSet<T> {
    fn len(&self) -> usize {
        BTreeSet::len(self)
    }
}

impl<K, V> Len for HashMap<K, V> {
    fn len(&self) -> usize {
        HashMap::len(self)
    }
}

impl<T> Len for HashSet<T> {
    fn len(&self) -> usize {
        HashSet::len(self)
    }
}

impl<T> Len for LinkedList<T> {
    fn len(&self) -> usize {
        LinkedList::len(self)
    }
}

impl<T> Len for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<T> Len for VecDeque<T> {
    fn len(&self) -> usize {
        VecDeque::len(self)
    }
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
pub struct Parallel<T: Send + Sync + Clone + 'static> {
    options: ParallelOptions,
    started: Arc<Mutex<bool>>,
    finished: Arc<Mutex<bool>>,
    finished_noti: Arc<Notify>,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    running: Arc<AtomicUsize>,
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + Clone> Parallel<T> {
    pub fn new() -> Self {
        let options: ParallelOptions = Default::default();
        Parallel {
            options,
            started: Arc::new(Mutex::new(false)),
            finished: Arc::new(Mutex::new(false)),
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
            finished: Arc::new(Mutex::new(false)),
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

    pub fn running(&self) -> usize {
        self.running.load(Ordering::SeqCst)
    }

    fn set_running(&self, value: usize) {
        self.running.store(value, Ordering::SeqCst);
    }

    fn dec_running(&self, td: &impl TaskDelegationBase<Parallel<T>, T>) {
        self.running.fetch_sub(1, Ordering::SeqCst);
        self.check_finished(td);
    }

    fn check_finished(&self, td: &impl TaskDelegationBase<Parallel<T>, T>) {
        if self.running() == 0 {
            let mut finished = self.finished.lock().unwrap();
            *finished = true;
            td.on_finished(self);
            self.set_started(false);
            self.finished_noti.notify_one();
        }
    }

    pub fn start<
        I: IntoParallelIterator<Item = T> + Len + Send,
        TD: TaskDelegation<Parallel<T>, T> + Send + Sync + Clone + 'static,
    >(
        &self,
        collection: I,
        delegate: &TD,
    ) {
        if self.is_cancelled() {
            panic!("Queue is already cancelled.")
        }

        if !self.set_started(true) {
            return;
        }

        self.set_running(collection.len());
        delegate.on_started(self);

        let pool = ThreadPoolBuilder::new()
            .num_threads(self.options.threads)
            .build()
            .unwrap();
        let delegate = delegate.clone();
        let this = self.clone();

        if self.options.threshold.is_zero() {
            pool.install(move || {
                collection.into_par_iter().for_each(|item| {
                    while !this.is_cancelled() && this.is_paused() {
                        thread::sleep(this.options.pause_timeout);
                    }

                    if this.is_cancelled() {
                        this.dec_running(&delegate);
                        return;
                    }

                    if let Ok(result) = delegate.process(&this, &item) {
                        if !delegate.on_completed(&this, &item, &result) {
                            this.cancel();
                            return;
                        }
                    }

                    this.dec_running(&delegate);
                });

                drop(delegate);
                drop(this);
            });

            return;
        }

        pool.install(move || {
            collection.into_par_iter().for_each(|item| {
                while !this.is_cancelled() && this.is_paused() {
                    thread::sleep(this.options.pause_timeout);
                }

                if this.is_cancelled() {
                    this.dec_running(&delegate);
                    return;
                }

                if let Ok(result) = delegate.process(&this, &item) {
                    let time = Instant::now();

                    if !delegate.on_completed(&this, &item, &result) {
                        this.cancel();
                        return;
                    }

                    if !this.options.threshold.is_zero() && time.elapsed() < this.options.threshold
                    {
                        let remaining = this.options.threshold - time.elapsed();
                        thread::sleep(remaining);
                    }
                }

                this.dec_running(&delegate);
            });

            drop(delegate);
            drop(this);
        });
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

    pub fn wait(&self) {
        block_on(self.finished_noti.notified());
    }

    pub async fn wait_async(&self) {
        self.finished_noti.notified().await;
    }

    pub fn wait_for(&self, timeout: Duration) -> bool {
        if timeout.is_zero() {
            self.wait();
            return true;
        }

        let start = Instant::now();
        let finished = self.finished.lock().unwrap();

        while !*finished && start.elapsed() < timeout {
            let wait_timeout = timeout - start.elapsed();
            let pause_timeout = self.options.pause_timeout.min(wait_timeout);
            let result = block_on(tokio_timeout(pause_timeout, self.finished_noti.notified()));
            if result.is_err() {
                break;
            }
        }

        start.elapsed() < timeout
    }

    pub async fn wait_for_async(&self, timeout: Duration) -> bool {
        if timeout.is_zero() {
            self.wait();
            return true;
        }

        let start = Instant::now();
        let finished = self.finished.lock().unwrap();

        while !*finished && start.elapsed() < timeout {
            let wait_timeout = timeout - start.elapsed();
            let pause_timeout = self.options.pause_timeout.min(wait_timeout);

            select! {
                _ = self.finished_noti.notified() => {},
                _ = tokio_sleep(pause_timeout) => {}
            }
        }

        start.elapsed() < timeout
    }
}
