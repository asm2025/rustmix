// use crossbeam::deque::{Injector, Steal, Stealer, Worker};
// use std::{
//     sync::{
//         atomic::{AtomicBool, AtomicUsize, Ordering},
//         Arc, Mutex,
//     },
//     thread,
// };
// use tokio::{
//     sync::Notify,
//     time::{Duration, Instant},
// };

// use super::{cond::Mutcond, *};
// use crate::{error::*, Result};

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct InjectorWorkerOptions {
//     pub behavior: QueueBehavior,
//     pub threads: usize,
//     pub threshold: Duration,
//     pub sleep_after_send: Duration,
//     pub pause_timeout: Duration,
// }

// impl Default for InjectorWorkerOptions {
//     fn default() -> Self {
//         InjectorWorkerOptions {
//             behavior: QUEUE_BEHAVIOR_DEF,
//             threads: THREADS_DEF.clamp(THREADS_MIN, THREADS_MAX),
//             threshold: THRESHOLD_DEF,
//             sleep_after_send: SLEEP_AFTER_SEND_DEF,
//             pause_timeout: PAUSE_TIMEOUT_DEF.clamp(PAUSE_TIMEOUT_MIN, PAUSE_TIMEOUT_MAX),
//         }
//     }
// }

// impl InjectorWorkerOptions {
//     pub fn new() -> Self {
//         Default::default()
//     }

//     pub fn with_behavior(&self, behavior: QueueBehavior) -> Self {
//         InjectorWorkerOptions {
//             behavior,
//             ..self.clone()
//         }
//     }

//     pub fn with_threads(&self, threads: usize) -> Self {
//         InjectorWorkerOptions {
//             threads: threads.clamp(THREADS_MIN, THREADS_MAX),
//             ..self.clone()
//         }
//     }

//     pub fn with_threshold(&self, threshold: Duration) -> Self {
//         InjectorWorkerOptions {
//             threshold,
//             ..self.clone()
//         }
//     }

//     pub fn with_sleep_after_send(&self, sleep_after_send: Duration) -> Self {
//         InjectorWorkerOptions {
//             sleep_after_send,
//             ..self.clone()
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct InjectorWorker<T: StaticTaskItem, H: TaskDelegation<T>, S: StaticTaskItem> {
//     pub options: InjectorWorkerOptions,
//     pub state: Arc<Mutex<S>>,
//     injector: Arc<Injector<T>>,
//     stealers: Arc<Mutex<Vec<Stealer<T>>>>,
//     handler: Arc<RwLock<H>>,
//     started: Arc<Mutex<bool>>,
//     finished: Arc<AtomicBool>,
//     finished_cond: Arc<Mutcond>,
//     finished_noti: Arc<Notify>,
//     completed: Arc<AtomicBool>,
//     paused: Arc<AtomicBool>,
//     cancelled: Arc<AtomicBool>,
//     workers: Arc<AtomicUsize>,
//     running: Arc<AtomicUsize>,
// }

// impl<T: StaticTaskItem, H: TaskDelegation<T>, S: StaticTaskItem> InjectorWorker<T, H, S> {
//     pub fn new(handler: H, state: S) -> Self {
//         InjectorWorker {
//             options: Default::default(),
//             state: Arc::new(Mutex::new(state)),
//             injector: Arc::new(Injector::new()),
//             stealers: Arc::new(Mutex::new(Vec::new())),
//             handler: Arc::new(RwLock::new(handler)),
//             started: Arc::new(Mutex::new(false)),
//             finished: Arc::new(AtomicBool::new(false)),
//             finished_cond: Arc::new(Mutcond::new()),
//             finished_noti: Arc::new(Notify::new()),
//             completed: Arc::new(AtomicBool::new(false)),
//             paused: Arc::new(AtomicBool::new(false)),
//             cancelled: Arc::new(AtomicBool::new(false)),
//             workers: Arc::new(AtomicUsize::new(0)),
//             running: Arc::new(AtomicUsize::new(0)),
//         }
//     }

//     pub fn with_options(handler: H, state: S, options: InjectorWorkerOptions) -> Self {
//         InjectorWorker {
//             options,
//             state: Arc::new(Mutex::new(state)),
//             injector: Arc::new(Injector::new()),
//             stealers: Arc::new(Mutex::new(Vec::new())),
//             handler: Arc::new(RwLock::new(handler)),
//             started: Arc::new(Mutex::new(false)),
//             finished: Arc::new(AtomicBool::new(false)),
//             finished_cond: Arc::new(Mutcond::new()),
//             finished_noti: Arc::new(Notify::new()),
//             completed: Arc::new(AtomicBool::new(false)),
//             paused: Arc::new(AtomicBool::new(false)),
//             cancelled: Arc::new(AtomicBool::new(false)),
//             workers: Arc::new(AtomicUsize::new(0)),
//             running: Arc::new(AtomicUsize::new(0)),
//         }
//     }

//     pub fn is_started(&self) -> bool {
//         *self.started.lock().unwrap()
//     }

//     fn set_started(&self, value: bool) -> bool {
//         let mut started = self.started.lock().unwrap();

//         if *started && value {
//             return false;
//         }

//         *started = true;
//         true
//     }

//     pub fn is_completed(&self) -> bool {
//         self.completed.load(Ordering::SeqCst)
//     }

//     pub fn is_paused(&self) -> bool {
//         self.paused.load(Ordering::SeqCst)
//     }

//     pub fn is_cancelled(&self) -> bool {
//         self.cancelled.load(Ordering::SeqCst)
//     }

//     pub fn is_finished(&self) -> bool {
//         self.finished.load(Ordering::SeqCst)
//     }

//     pub fn is_busy(&self) -> bool {
//         self.len() + self.running.load(Ordering::SeqCst) > 0
//     }

//     pub fn is_empty(&self) -> bool {
//         self.len() == 0
//     }

//     pub fn len(&self) -> usize {
//         self.injector.len()
//     }

//     pub fn workers(&self) -> usize {
//         self.workers.load(Ordering::SeqCst)
//     }

//     fn set_workers(&self, value: usize) {
//         self.workers.store(value, Ordering::SeqCst);
//     }

//     fn dec_workers(&self) {
//         self.workers.fetch_sub(1, Ordering::SeqCst);
//         self.check_finished();
//     }

//     fn check_finished(&self) {
//         if self.workers() > 0 || (!self.is_completed() && !self.is_cancelled()) {
//             return;
//         }

//         self.completed.store(true, Ordering::SeqCst);
//         self.finished.store(true, Ordering::SeqCst);
//         {
//             let handler = self.handler.read().unwrap();

//             if self.is_cancelled() {
//                 handler.on_cancelled();
//             } else {
//                 handler.on_finished();
//             }
//         }
//         self.set_started(false);
//         self.finished_cond.notify_one();
//         self.finished_noti.notify_one();
//     }

//     pub fn running(&self) -> usize {
//         self.running.load(Ordering::SeqCst)
//     }

//     fn inc_running(&self) {
//         self.running.fetch_add(1, Ordering::SeqCst);
//     }

//     fn dec_running(&self) {
//         self.running.fetch_sub(1, Ordering::SeqCst);
//     }

//     pub fn start(&self) -> Result<()> {
//         if self.is_cancelled() {
//             return Err(CancelledError.into());
//         }

//         if self.is_completed() && self.is_empty() {
//             return Err(QueueCompletedError.into());
//         }

//         if !self.set_started(true) {
//             return Err(QueueStartedError.into());
//         }

//         self.set_workers(self.options.threads);
//         {
//             let handler = self.handler.read().unwrap();
//             handler.on_started();
//         }
//         let mut mutstealers = self.stealers.lock()?;

//         for _ in 0..self.options.threads {
//             let worker = if self.options.behavior == QueueBehavior::LIFO {
//                 Worker::<T>::new_lifo()
//             } else {
//                 Worker::<T>::new_fifo()
//             };
//             let stealer = worker.stealer();
//             mutstealers.push(stealer);
//             let this = self.clone();
//             let global = self.injector.clone();
//             let local = Arc::new(Mutex::new(worker));
//             let stealers = self.stealers.clone();
//             thread::spawn(move || {
//                 let handler = this.handler.read().unwrap();

//                 if this.options.threshold.is_zero() {
//                     loop {
//                         if this.is_cancelled() || (this.is_empty() && this.is_completed()) {
//                             break;
//                         }

//                         if this.is_paused() {
//                             thread::sleep(this.options.pause_timeout);
//                             continue;
//                         }

//                         let Some(item) = this.dequeue_wait(&global, &local, &stealers) else {
//                             continue;
//                         };
//                         this.inc_running();
//                         match handler.process(&item) {
//                             Ok(it) => {
//                                 if !handler.on_completed(&item, &it) {
//                                     this.dec_running();
//                                     break;
//                                 }
//                             }
//                             Err(e) => {
//                                 if !handler.on_completed(&item, &TaskResult::Error(e.get_message()))
//                                 {
//                                     this.dec_running();
//                                     break;
//                                 }
//                             }
//                         }
//                         this.dec_running();
//                     }

//                     this.dec_workers();
//                     return;
//                 }

//                 loop {
//                     if this.is_cancelled() || (this.is_empty() && this.is_completed()) {
//                         break;
//                     }

//                     if this.is_paused() {
//                         thread::sleep(this.options.pause_timeout);
//                         continue;
//                     }

//                     let Some(item) = this.dequeue_wait(&global, &local, &stealers) else {
//                         continue;
//                     };
//                     this.inc_running();
//                     match handler.process(&item) {
//                         Ok(it) => {
//                             if !handler.on_completed(&item, &it) {
//                                 this.dec_running();
//                                 break;
//                             }

//                             if !this.options.threshold.is_zero() {
//                                 let time = Instant::now();

//                                 if time.elapsed() < this.options.threshold {
//                                     let remaining = this.options.threshold - time.elapsed();
//                                     thread::sleep(remaining);
//                                 }
//                             }
//                         }
//                         Err(e) => {
//                             if !handler.on_completed(&item, &TaskResult::Error(e.get_message())) {
//                                 this.dec_running();
//                                 break;
//                             }
//                         }
//                     }
//                     this.dec_running();
//                 }

//                 this.dec_workers();
//             });
//         }

//         Ok(())
//     }

//     pub async fn start_async(&self) -> Result<()> {
//         if self.is_cancelled() {
//             return Err(CancelledError.into());
//         }

//         if self.is_completed() && self.is_empty() {
//             return Err(QueueCompletedError.into());
//         }

//         if !self.set_started(true) {
//             return Err(QueueStartedError.into());
//         }

//         self.set_workers(self.options.threads);
//         {
//             let handler = self.handler.read().unwrap();
//             handler.on_started();
//         }
//         let mut mutstealers = self.stealers.lock().unwrap();

//         for _ in 0..self.options.threads {
//             let worker = if self.options.behavior == QueueBehavior::LIFO {
//                 Worker::<T>::new_lifo()
//             } else {
//                 Worker::<T>::new_fifo()
//             };
//             let stealer = worker.stealer();
//             mutstealers.push(stealer);
//             let this = self.clone();
//             let global = self.injector.clone();
//             let local = Arc::new(Mutex::new(worker));
//             let stealers = self.stealers.clone();
//             tokio::spawn(async move {
//                 let handler = this.handler.read().unwrap();

//                 if this.options.threshold.is_zero() {
//                     loop {
//                         if this.is_cancelled() || (this.is_empty() && this.is_completed()) {
//                             break;
//                         }

//                         if this.is_paused() {
//                             thread::sleep(this.options.pause_timeout);
//                             continue;
//                         }

//                         let Some(item) = this.dequeue_wait(&global, &local, &stealers) else {
//                             continue;
//                         };
//                         this.inc_running();
//                         match handler.process_async(&item).await {
//                             Ok(it) => {
//                                 if !handler.on_completed(&item, &it) {
//                                     this.dec_running();
//                                     break;
//                                 }
//                             }
//                             Err(e) => {
//                                 if !handler.on_completed(&item, &TaskResult::Error(e.get_message()))
//                                 {
//                                     this.dec_running();
//                                     break;
//                                 }
//                             }
//                         }
//                         this.dec_running();
//                     }

//                     this.dec_workers();
//                     return;
//                 }

//                 loop {
//                     if this.is_cancelled() || (this.is_empty() && this.is_completed()) {
//                         break;
//                     }

//                     if this.is_paused() {
//                         thread::sleep(this.options.pause_timeout);
//                         continue;
//                     }

//                     let Some(item) = this.dequeue_wait(&global, &local, &stealers) else {
//                         continue;
//                     };
//                     this.inc_running();
//                     match handler.process_async(&item).await {
//                         Ok(it) => {
//                             if !handler.on_completed(&item, &it) {
//                                 this.dec_running();
//                                 break;
//                             }

//                             if !this.options.threshold.is_zero() {
//                                 let time = Instant::now();

//                                 if time.elapsed() < this.options.threshold {
//                                     let remaining = this.options.threshold - time.elapsed();
//                                     thread::sleep(remaining);
//                                 }
//                             }
//                         }
//                         Err(e) => {
//                             if !handler.on_completed(&item, &TaskResult::Error(e.get_message())) {
//                                 this.dec_running();
//                                 break;
//                             }
//                         }
//                     }
//                     this.dec_running();
//                 }

//                 this.dec_workers();
//             });
//         }

//         Ok(())
//     }

//     pub fn enqueue(&self, item: T) -> Result<()> {
//         if self.is_cancelled() {
//             return Err(CancelledError.into());
//         }

//         if self.is_completed() {
//             return Err(QueueCompletedError.into());
//         }

//         self.injector.push(item);

//         if !self.options.sleep_after_send.is_zero() {
//             thread::sleep(self.options.sleep_after_send);
//         }

//         Ok(())
//     }

//     pub fn dequeue(
//         &self,
//         global: &Arc<Injector<T>>,
//         local: &Arc<Mutex<Worker<T>>>,
//         stealers: &Arc<Mutex<Vec<Stealer<T>>>>,
//     ) -> Option<T> {
//         self.deq(false, global, local, stealers)
//     }

//     pub fn dequeue_wait(
//         &self,
//         global: &Arc<Injector<T>>,
//         local: &Arc<Mutex<Worker<T>>>,
//         stealers: &Arc<Mutex<Vec<Stealer<T>>>>,
//     ) -> Option<T> {
//         self.deq(true, global, local, stealers)
//     }

//     fn deq(
//         &self,
//         wait_for_item: bool,
//         global: &Arc<Injector<T>>,
//         local: &Arc<Mutex<Worker<T>>>,
//         stealers: &Arc<Mutex<Vec<Stealer<T>>>>,
//     ) -> Option<T> {
//         let local = local.lock().unwrap();
//         // Pop a task from the local queue, if not empty.
//         local.pop().or_else(|| {
//             // Otherwise, we need to look for a task elsewhere.
//             if self.is_cancelled() {
//                 return None;
//             }

//             if wait_for_item && self.is_paused() {
//                 thread::sleep(self.options.pause_timeout);
//                 return None;
//             }

//             // Try stealing a batch of tasks from the global queue.
//             global
//                 .steal_batch_with_limit_and_pop(&local, 10)
//                 // Or try stealing a task from one of the other threads.
//                 .or_else(|| {
//                     stealers
//                         .lock()
//                         .unwrap()
//                         .iter()
//                         .map(|s| s.steal_batch_with_limit_and_pop(&local, 10))
//                         .find(|s| s.is_success())
//                         .unwrap_or_else(|| Steal::Empty)
//                 })
//                 .success()
//         })
//     }

//     pub fn stop(&self, enforce: bool) {
//         if enforce {
//             self.cancel();
//         } else {
//             self.complete();
//         }
//     }

//     pub fn complete(&self) {
//         self.completed.store(true, Ordering::SeqCst);
//     }

//     pub fn cancel(&self) {
//         self.cancelled.store(true, Ordering::SeqCst);
//     }

//     pub fn pause(&self) {
//         self.paused.store(true, Ordering::SeqCst);
//     }

//     pub fn resume(&self) {
//         self.paused.store(false, Ordering::SeqCst);
//     }

//     pub fn wait(&self) -> Result<()> {
//         wait(self, &self.finished_cond)
//     }

//     pub async fn wait_async(&self) -> Result<()> {
//         wait_async(self, &self.finished_noti).await
//     }

//     pub fn wait_until(&self, cond: impl Fn(&InjectorWorker<T, H, S>) -> bool) -> Result<()> {
//         wait_until(self, &self.finished_cond, cond)
//     }

//     pub async fn wait_until_async<
//         F: Fn(&InjectorWorker<T, H, S>) -> Pin<Box<dyn Future<Output = bool> + Send>>,
//     >(
//         &self,
//         cond: F,
//     ) -> Result<()> {
//         wait_until_async(self, &self.finished_noti, cond).await
//     }

//     pub fn wait_for(&self, timeout: Duration) -> Result<()> {
//         wait_for(self, timeout, &self.finished_cond)
//     }

//     pub async fn wait_for_async(&self, timeout: Duration) -> Result<()> {
//         wait_for_async(self, timeout, &self.finished_noti).await
//     }

//     pub fn wait_for_until(
//         &self,
//         timeout: Duration,
//         cond: impl Fn(&InjectorWorker<T, H, S>) -> bool,
//     ) -> Result<()> {
//         wait_for_until(self, timeout, &self.finished_cond, cond)
//     }

//     pub async fn wait_for_until_async<
//         F: Fn(&InjectorWorker<T, H, S>) -> Pin<Box<dyn Future<Output = bool> + Send>>,
//     >(
//         &self,
//         timeout: Duration,
//         cond: F,
//     ) -> Result<()> {
//         wait_for_until_async(self, timeout, &self.finished_noti, cond).await
//     }
// }

// impl<T: StaticTaskItem, H: TaskDelegation<T>, S: StaticTaskItem> AwaitableConsumer<T>
//     for InjectorWorker<T, H, S>
// {
//     fn is_cancelled(&self) -> bool {
//         self.is_cancelled()
//     }

//     fn is_finished(&self) -> bool {
//         self.is_finished()
//     }
// }
