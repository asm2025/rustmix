use crossbeam::queue::SegQueue;
use std::{
    mem,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
};
use tokio::{
    sync::Notify,
    time::{Duration, Instant},
};

use super::{cond::Mutcond, *};
use crate::{error::*, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerOptions {
    pub threads: usize,
    pub threshold: Duration,
    pub sleep_after_send: Duration,
    pub peek_timeout: Duration,
    pub pause_timeout: Duration,
}

impl Default for ConsumerOptions {
    fn default() -> Self {
        ConsumerOptions {
            threads: THREADS_DEF.clamp(THREADS_MIN, THREADS_MAX),
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

    pub fn with_threads(&self, threads: usize) -> Self {
        ConsumerOptions {
            threads: threads.clamp(THREADS_MIN, THREADS_MAX),
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
pub struct Consumer<T: StaticTaskItem> {
    pub options: ConsumerOptions,
    items: Arc<SegQueue<T>>,
    items_cond: Arc<Mutcond>,
    started: Arc<Mutex<bool>>,
    finished: Arc<AtomicBool>,
    finished_cond: Arc<Mutcond>,
    finished_noti: Arc<Notify>,
    completed: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    consumers: Arc<AtomicUsize>,
    running: Arc<AtomicUsize>,
}

impl<T: StaticTaskItem> Consumer<T> {
    pub fn new() -> Self {
        Consumer {
            options: Default::default(),
            items: Arc::new(SegQueue::new()),
            items_cond: Arc::new(Mutcond::new()),
            started: Arc::new(Mutex::new(false)),
            finished: Arc::new(AtomicBool::new(false)),
            finished_cond: Arc::new(Mutcond::new()),
            finished_noti: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            consumers: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn with_options(options: ConsumerOptions) -> Self {
        Consumer {
            options,
            items: Arc::new(SegQueue::new()),
            items_cond: Arc::new(Mutcond::new()),
            started: Arc::new(Mutex::new(false)),
            finished: Arc::new(AtomicBool::new(false)),
            finished_cond: Arc::new(Mutcond::new()),
            finished_noti: Arc::new(Notify::new()),
            completed: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            consumers: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn is_started(&self) -> bool {
        *self.started.lock().unwrap()
    }

    fn set_started(&self, value: bool) -> bool {
        let mut started = self.started.lock().unwrap();
        *started = value;
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

    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::SeqCst)
    }

    pub fn is_busy(&self) -> bool {
        self.len() + self.running.load(Ordering::SeqCst) > 0
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn consumers(&self) -> usize {
        self.consumers.load(Ordering::SeqCst)
    }

    fn set_consumers(&self, value: usize) {
        self.consumers.store(value, Ordering::SeqCst);
    }

    fn dec_consumers(&self) -> bool {
        self.consumers.fetch_sub(1, Ordering::SeqCst);
        self.consumers() == 0 && (self.is_completed() || self.is_cancelled())
    }

    fn finish(&self) {
        if !self.is_completed() && !self.is_cancelled() {
            return;
        }

        self.completed.store(true, Ordering::SeqCst);
        self.finished.store(true, Ordering::SeqCst);
        self.set_started(false);
        self.finished_cond.notify_all();
        self.finished_noti.notify_waiters();
        thread::sleep(Duration::ZERO);
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

    pub fn start<H: TaskDelegation<Consumer<T>, T>>(&self, handler: &H) -> Result<()> {
        if self.is_cancelled() {
            return Err(CanceledError.into());
        }

        if self.is_completed() && self.is_empty() {
            return Err(QueueCompletedError.into());
        }

        if !self.set_started(true) {
            return Err(QueueStartedError.into());
        }

        self.set_consumers(self.options.threads);
        handler.on_started(self);

        for _ in 0..self.options.threads {
            let this = self.clone();
            let handler = handler.clone();
            thread::spawn(move || {
                if this.options.threshold.is_zero() {
                    loop {
                        if this.is_cancelled() || (!this.is_busy() && this.is_completed()) {
                            break;
                        }

                        if this.is_paused() {
                            thread::sleep(this.options.pause_timeout);
                            continue;
                        }

                        let Some(item) = this.dequeue_wait() else {
                            continue;
                        };
                        this.inc_running();
                        match handler.process(&this, &item) {
                            Ok(it) => {
                                if !handler.on_completed(&this, &item, &it) {
                                    this.dec_running();
                                    break;
                                }
                            }
                            Err(e) => {
                                if !handler.on_completed(
                                    &this,
                                    &item,
                                    &TaskResult::Error(e.get_message()),
                                ) {
                                    this.dec_running();
                                    break;
                                }
                            }
                        }
                        this.dec_running();
                    }

                    if !this.dec_consumers() {
                        return;
                    }

                    if this.is_cancelled() {
                        handler.on_cancelled(&this);
                    } else {
                        handler.on_finished(&this);
                    }

                    this.finish();
                    return;
                }

                loop {
                    if this.is_cancelled() || (!this.is_busy() && this.is_completed()) {
                        break;
                    }

                    if this.is_paused() {
                        thread::sleep(this.options.pause_timeout);
                        continue;
                    }

                    let Some(item) = this.dequeue_wait() else {
                        continue;
                    };
                    this.inc_running();
                    match handler.process(&this, &item) {
                        Ok(it) => {
                            if !handler.on_completed(&this, &item, &it) {
                                this.dec_running();
                                break;
                            }

                            if !this.options.threshold.is_zero() {
                                let time = Instant::now();

                                if time.elapsed() < this.options.threshold {
                                    let remaining = this.options.threshold - time.elapsed();
                                    thread::sleep(remaining);
                                }
                            }
                        }
                        Err(e) => {
                            if !handler.on_completed(
                                &this,
                                &item,
                                &TaskResult::Error(e.get_message()),
                            ) {
                                this.dec_running();
                                break;
                            }
                        }
                    }
                    this.dec_running();
                }

                if !this.dec_consumers() {
                    return;
                }

                if this.is_cancelled() {
                    handler.on_cancelled(&this);
                } else {
                    handler.on_finished(&this);
                }

                this.finish();
            });
        }

        Ok(())
    }

    pub fn enqueue(&self, item: T) -> Result<()> {
        if self.is_cancelled() {
            return Err(CanceledError.into());
        }

        if self.is_completed() {
            return Err(QueueCompletedError.into());
        }

        self.items.push(item);

        if !self.options.sleep_after_send.is_zero() {
            thread::sleep(self.options.sleep_after_send);
        }

        self.items_cond.notify_all();
        Ok(())
    }

    pub fn dequeue(&self) -> Option<T> {
        self.deq(false)
    }

    pub fn dequeue_wait(&self) -> Option<T> {
        self.deq(true)
    }

    fn deq(&self, wait_for_item: bool) -> Option<T> {
        if wait_for_item {
            while self.items.is_empty() && !self.is_cancelled() && !self.is_completed() {
                if !self
                    .items_cond
                    .wait_timeout(self.options.peek_timeout)
                    .unwrap()
                {
                    continue;
                }

                if self.is_cancelled() || self.is_completed() {
                    return None;
                }

                return self.items.pop();
            }
        }

        if self.items.is_empty() || self.is_cancelled() {
            return None;
        }

        self.items.pop()
    }

    pub fn clear(&mut self) {
        self.items = mem::replace(&mut self.items, Arc::new(SegQueue::new()));
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

    pub fn wait(&self) -> Result<()> {
        wait(self, &self.finished_cond)
    }

    pub async fn wait_async(&self) -> Result<()> {
        wait_async(self, &self.finished_noti).await
    }

    pub fn wait_until(&self, cond: impl Fn(&Consumer<T>) -> bool) -> Result<()> {
        wait_until(self, &self.finished_cond, cond)
    }

    pub async fn wait_until_async(
        &self,
        cond: impl Fn(&Consumer<T>) -> Pin<Box<dyn Future<Output = bool> + Send>>,
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
        cond: impl Fn(&Consumer<T>) -> bool,
    ) -> Result<()> {
        wait_for_until(self, timeout, &self.finished_cond, cond)
    }

    pub async fn wait_for_until_async<
        F: Fn(&Consumer<T>) -> Pin<Box<dyn Future<Output = bool> + Send>>,
    >(
        &self,
        timeout: Duration,
        cond: F,
    ) -> Result<()> {
        wait_for_until_async(self, timeout, &self.finished_noti, cond).await
    }
}

impl<T: StaticTaskItem> AwaitableConsumer<T> for Consumer<T> {
    fn is_cancelled(&self) -> bool {
        Consumer::is_cancelled(self)
    }

    fn is_finished(&self) -> bool {
        Consumer::is_finished(self)
    }
}
