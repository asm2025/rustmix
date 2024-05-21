use crossbeam::channel;
use std::{
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
pub struct ProducerConsumerOptions {
    pub capacity: usize,
    pub threads: usize,
    pub threshold: Duration,
    pub sleep_after_send: Duration,
    pub peek_timeout: Duration,
    pub pause_timeout: Duration,
}

impl Default for ProducerConsumerOptions {
    fn default() -> Self {
        ProducerConsumerOptions {
            capacity: CAPACITY_DEF,
            threads: THREADS_DEF.clamp(THREADS_MIN, THREADS_MAX),
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

    pub fn with_threads(&self, threads: usize) -> Self {
        ProducerConsumerOptions {
            threads: threads.clamp(THREADS_MIN, THREADS_MAX),
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
pub struct ProducerConsumer<T: StaticTaskItem> {
    options: ProducerConsumerOptions,
    started: Arc<Mutex<bool>>,
    finished: Arc<AtomicBool>,
    finished_cond: Arc<Mutcond>,
    finished_noti: Arc<Notify>,
    completed: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    consumers: Arc<AtomicUsize>,
    running: Arc<AtomicUsize>,
    sender: channel::Sender<T>,
    receiver: channel::Receiver<T>,
}

impl<T: StaticTaskItem> ProducerConsumer<T> {
    pub fn new() -> Self {
        let options: ProducerConsumerOptions = Default::default();
        let (sender, receiver) = channel::bounded::<T>(options.capacity);
        ProducerConsumer {
            options,
            sender,
            receiver,
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

    pub fn with_options(options: ProducerConsumerOptions) -> Self {
        let (sender, receiver) = channel::bounded::<T>(options.capacity);
        ProducerConsumer {
            options,
            sender,
            receiver,
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
        self.sender.len() + self.receiver.len()
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

    pub fn start<H: TaskDelegation<ProducerConsumer<T>, T>>(&self, handler: &H) -> Result<()> {
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

                        let Ok(item) = this.receiver.recv_timeout(this.options.peek_timeout) else {
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
                    if this.is_cancelled() || (this.is_busy() && this.is_completed()) {
                        break;
                    }

                    if this.is_paused() {
                        thread::sleep(this.options.pause_timeout);
                        continue;
                    }

                    let Ok(item) = this.receiver.recv_timeout(this.options.peek_timeout) else {
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

        self.sender.send(item)?;

        if !self.options.sleep_after_send.is_zero() {
            thread::sleep(self.options.sleep_after_send);
        }

        Ok(())
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

    pub fn wait(&self) -> Result<()> {
        wait(self, &self.finished_cond)
    }

    pub async fn wait_async(&self) -> Result<()> {
        wait_async(self, &self.finished_noti).await
    }

    pub fn wait_until(&self, cond: impl Fn(&ProducerConsumer<T>) -> bool) -> Result<()> {
        wait_until(self, &self.finished_cond, cond)
    }

    pub async fn wait_until_async(
        &self,
        cond: impl Fn(&ProducerConsumer<T>) -> Pin<Box<dyn Future<Output = bool> + Send>>,
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
        cond: impl Fn(&ProducerConsumer<T>) -> bool,
    ) -> Result<()> {
        wait_for_until(self, timeout, &self.finished_cond, cond)
    }

    pub async fn wait_for_until_async<
        F: Fn(&ProducerConsumer<T>) -> Pin<Box<dyn Future<Output = bool> + Send>>,
    >(
        &self,
        timeout: Duration,
        cond: F,
    ) -> Result<()> {
        wait_for_until_async(self, timeout, &self.finished_noti, cond).await
    }
}

impl<T: StaticTaskItem> AwaitableConsumer<T> for ProducerConsumer<T> {
    fn is_cancelled(&self) -> bool {
        ProducerConsumer::is_cancelled(self)
    }

    fn is_finished(&self) -> bool {
        ProducerConsumer::is_finished(self)
    }
}
