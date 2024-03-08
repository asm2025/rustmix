// use crossbeam::deque::{Injector, Stealer, Worker};
// use std::{
//     error::Error,
//     future::Future,
//     iter,
//     sync::{
//         atomic::{AtomicBool, AtomicUsize},
//         Arc, Condvar, Mutex,
//     },
//     thread,
//     time::{Duration, Instant},
// };
// use tokio::sync::Notify;

// use super::*;

// pub trait InjectorConsumerDelegation<T: Send + Clone + 'static> {
//     fn process_task(&self, pc: &InjectorConsumer<T>, item: T)
//         -> Result<TaskResult, Box<dyn Error>>;
//     fn on_task_completed(&self, pc: &InjectorConsumer<T>, item: T, result: TaskResult) -> bool;
//     fn on_finished(&self, pc: &InjectorConsumer<T>);
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct InjectorConsumerOptions {
//     pub threads: usize,
//     pub capacity: usize,
//     pub threshold: Duration,
//     pub sleep_after_enqueue: Duration,
//     pub pause_timeout: Duration,
// }

// impl Default for InjectorConsumerOptions {
//     fn default() -> Self {
//         InjectorConsumerOptions {
//             capacity: CAPACITY_DEF,
//             threads: THREADS_DEF,
//             threshold: THRESHOLD_DEF,
//             sleep_after_enqueue: SLEEP_AFTER_ENQUEUE_DEF,
//             pause_timeout: PAUSE_TIMEOUT_DEF.clamp(PAUSE_TIMEOUT_MIN, PAUSE_TIMEOUT_MAX),
//         }
//     }
// }

// impl InjectorConsumerOptions {
//     pub fn new() -> Self {
//         Default::default()
//     }

//     pub fn with_capacity(self, capacity: usize) -> Self {
//         InjectorConsumerOptions { capacity, ..self }
//     }

//     pub fn with_threads(self, threads: usize) -> Self {
//         InjectorConsumerOptions {
//             threads: if threads > 0 { threads } else { 1 },
//             ..self
//         }
//     }

//     pub fn with_threshold(self, threshold: Duration) -> Self {
//         InjectorConsumerOptions { threshold, ..self }
//     }

//     pub fn with_sleep_after_enqueue(self, sleep_after_enqueue: Duration) -> Self {
//         InjectorConsumerOptions {
//             sleep_after_enqueue,
//             ..self
//         }
//     }
// }

// #[derive(Clone, Debug)]
// pub struct InjectorConsumer<T: Send + Clone + 'static> {
//     options: InjectorConsumerOptions,
//     injector: Injector<T>,
//     consumers: Vec<Worker<T>>,
//     started: Arc<Mutex<bool>>,
//     finished: Arc<Mutex<bool>>,
//     finished_cond: Arc<Condvar>,
//     finished_notify: Arc<Notify>,
//     completed: Arc<AtomicBool>,
//     paused: Arc<AtomicBool>,
//     cancelled: Arc<AtomicBool>,
//     running_count: Arc<AtomicUsize>,
// }

// impl<T: Send + Clone> InjectorConsumer<T> {
//     pub fn new() -> Self {
//         let options: InjectorConsumerOptions = Default::default();
//         InjectorConsumer {
//             options: options,
//             injector: Injector::new(),
//             consumers: Vec::new(),
//             started: Arc::new(Mutex::new(false)),
//             finished: Arc::new(Mutex::new(false)),
//             finished_cond: Arc::new(Condvar::new()),
//             finished_notify: Arc::new(Notify::new()),
//             completed: Arc::new(AtomicBool::new(false)),
//             paused: Arc::new(AtomicBool::new(false)),
//             cancelled: Arc::new(AtomicBool::new(false)),
//             running_count: Arc::new(AtomicUsize::new(0)),
//         }
//     }

//     pub fn with_options(options: InjectorConsumerOptions) -> Self {
//         InjectorConsumer {
//             options: options,
//             injector: Injector::new(),
//             consumers: Vec::new(),
//             paused: Arc::new(AtomicBool::new(false)),
//             cancelled: Arc::new(AtomicBool::new(false)),
//             running_count: Arc::new(AtomicUsize::new(0)),
//         }
//     }

//     fn find_task(
//         &self,
//         local: &Worker<T>,
//         global: &Injector<T>,
//         stealers: &[Stealer<T>],
//     ) -> Option<T> {
//         // Pop a task from the local queue, if not empty.
//         local.pop().or_else(|| {
//             // Otherwise, we need to look for a task elsewhere.
//             iter::repeat_with(|| {
//                 // Try stealing a batch of tasks from the global queue.
//                 global
//                     .steal_batch_with_limit_and_pop(local, 10)
//                     // Or try stealing a task from one of the other threads.
//                     .or_else(|| stealers.iter().map(|s| s.steal()).collect())
//             })
//             // Loop while no task was stolen and any steal operation needs to be retried.
//             .find(|s| !self.is_cancelled() && !s.is_retry())
//             // Extract the stolen task, if there is one.
//             .and_then(|s| s.success())
//         })
//     }

//     pub fn is_paused(&self) -> bool {
//         self.paused.load(std::sync::atomic::Ordering::Relaxed)
//     }

//     pub fn is_cancelled(&self) -> bool {
//         self.cancelled.load(std::sync::atomic::Ordering::Relaxed)
//     }

//     pub fn running(&self) -> usize {
//         self.running_count
//             .load(std::sync::atomic::Ordering::Relaxed)
//     }

//     fn inc_running(&self) {
//         self.running_count
//             .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
//     }

//     fn dec_running(&self) {
//         self.running_count
//             .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
//     }

//     pub fn start<S: InjectorConsumerDelegation<T> + Send + Clone + 'static>(
//         &self,
//         task_delegate: S,
//     ) {
//         if !self.consumers.is_empty() {
//             return;
//         }
//         let td = task_delegate.clone();
//         let builder = thread::Builder::new().name(format!("Producer {}", self.producers()));
//         builder
//             .spawn(move || {
//                 let producer = prod.lock().unwrap();

//                 loop {
//                     if producer.is_cancelled() || producer.is_completed() {
//                         break;
//                     }

//                     if producer.is_paused() {
//                         thread::sleep(producer.options.pause_timeout);
//                         continue;
//                     }

//                     if let Some(item) = match producer.options.behavior {
//                         QueueBehavior::FIFO => producer.dequeue(),
//                         QueueBehavior::LIFO => producer.pop(),
//                     } {
//                         if producer.is_cancelled() || producer.is_completed() {
//                             break;
//                         }

//                         producer.inc_running();
//                         producer.sender.send(item).unwrap();

//                         if !producer.options.sleep_after_enqueue.is_zero() {
//                             thread::sleep(producer.options.sleep_after_enqueue);
//                         }
//                     };
//                 }

//                 if !producer.is_cancelled() {
//                     while let Some(item) = match producer.options.behavior {
//                         QueueBehavior::FIFO => producer.dequeue(),
//                         QueueBehavior::LIFO => producer.pop(),
//                     } {
//                         if producer.is_cancelled() {
//                             break;
//                         }

//                         producer.inc_running();
//                         producer.sender.send(item).unwrap();

//                         if !producer.options.sleep_after_enqueue.is_zero() {
//                             thread::sleep(producer.options.sleep_after_enqueue);
//                         }
//                     }
//                 }

//                 producer.dec_producers(&td);
//             })
//             .unwrap();
//     }

//     pub fn start_consumer<S: ConsumerDelegation<T> + Send + Clone + 'static>(
//         &self,
//         task_delegate: S,
//     ) {
//         self.inc_consumers();
//         let cons = Arc::new(Mutex::new(self.clone()));
//         let td = task_delegate.clone();
//         let builder = thread::Builder::new().name(format!("Consumer {}", self.consumers()));
//         builder
//             .spawn(move || {
//                 let consumer = cons.lock().unwrap();

//                 loop {
//                     if consumer.is_cancelled()
//                         || (consumer.is_empty()
//                             && consumer.is_completed()
//                             && consumer.running() == 0)
//                     {
//                         break;
//                     }

//                     if consumer.is_paused() {
//                         thread::sleep(consumer.options.pause_timeout);
//                         continue;
//                     }

//                     let Ok(item) = consumer
//                         .receiver
//                         .recv_timeout(consumer.options.peek_timeout)
//                     else {
//                         continue;
//                     };

//                     if let Ok(result) = td.process_task(&consumer, item.clone()) {
//                         let time = Instant::now();

//                         if !td.on_task_completed(&consumer, item, result) {
//                             consumer.dec_running();
//                             break;
//                         }

//                         if !consumer.options.threshold.is_zero()
//                             && time.elapsed() < consumer.options.threshold
//                         {
//                             let remaining = consumer.options.threshold - time.elapsed();
//                             thread::sleep(remaining);
//                         }

//                         consumer.dec_running();
//                     }
//                 }

//                 consumer.dec_consumers(&td);
//             })
//             .unwrap();
//     }

//     pub fn stop(&self, enforce: bool) {
//         if enforce {
//             self.cancel();
//         } else {
//             self.complete();
//         }
//     }

//     pub fn enqueue(&self, item: T) {
//         let mut items = self.items.lock().unwrap();
//         items.push_back(item);
//         self.items_cond.notify_one();
//     }

//     fn dequeue(&self) -> Option<T> {
//         let mut items = self.items.lock().unwrap();

//         while items.is_empty() && !self.is_cancelled() && !self.is_completed() {
//             let result = self
//                 .items_cond
//                 .wait_timeout(items, self.options.peek_timeout)
//                 .unwrap();
//             items = result.0;

//             if result.1.timed_out() {
//                 continue;
//             }

//             if self.is_cancelled() || self.is_completed() {
//                 return None;
//             }

//             return items.pop_front();
//         }

//         if items.is_empty() || self.is_cancelled() {
//             return None;
//         }

//         items.pop_front()
//     }

//     fn pop(&self) -> Option<T> {
//         let mut items = self.items.lock().unwrap();

//         while items.is_empty() && !self.is_cancelled() && !self.is_completed() {
//             let result = self
//                 .items_cond
//                 .wait_timeout(items, self.options.peek_timeout)
//                 .unwrap();
//             items = result.0;

//             if result.1.timed_out() {
//                 continue;
//             }

//             if self.is_cancelled() || self.is_completed() {
//                 return None;
//             }

//             return items.pop_back();
//         }

//         if items.is_empty() || self.is_cancelled() {
//             return None;
//         }

//         items.pop_back()
//     }

//     pub fn peek(&self) -> Option<T> {
//         let items = self.items.lock().unwrap();

//         if items.is_empty() {
//             return None;
//         }

//         if let Some(item) = match self.options.behavior {
//             QueueBehavior::FIFO => items.front(),
//             QueueBehavior::LIFO => items.back(),
//         } {
//             Some(item.clone())
//         } else {
//             None
//         }
//     }

//     pub fn clear(&self) {
//         let mut items = self.items.lock().unwrap();
//         items.clear();
//     }

//     pub fn complete(&self) {
//         self.completed
//             .store(true, std::sync::atomic::Ordering::Relaxed);
//         self.items_cond.notify_all();
//     }

//     pub fn cancel(&self) {
//         self.cancelled
//             .store(true, std::sync::atomic::Ordering::Relaxed);
//         self.items_cond.notify_all();
//     }

//     pub fn pause(&self) {
//         self.paused
//             .store(true, std::sync::atomic::Ordering::Relaxed);
//     }

//     pub fn resume(&self) {
//         self.paused
//             .store(false, std::sync::atomic::Ordering::Relaxed);
//         self.items_cond.notify_all();
//     }

//     pub fn wait(&self) {
//         let finished = self.finished.lock().unwrap();

//         if !*finished {
//             let _ignored = self.finished_cond.wait(finished).unwrap();
//         }
//     }

//     pub async fn wait_async(&self) {
//         while !*self.finished.lock().unwrap() {
//             self.finished_notify.notified().await;
//             thread::sleep(self.options.pause_timeout);
//         }
//     }

//     pub fn wait_for(&self, timeout: Duration) -> bool {
//         if timeout == Duration::ZERO {
//             self.wait();
//             return true;
//         }

//         let start = Instant::now();
//         let mut finished = self.finished.lock().unwrap();

//         while !*finished && start.elapsed() < timeout {
//             let result = self
//                 .finished_cond
//                 .wait_timeout(finished, self.options.pause_timeout)
//                 .unwrap();
//             finished = result.0;
//             thread::sleep(self.options.pause_timeout);

//             if result.1.timed_out() || start.elapsed() >= timeout {
//                 return false;
//             }
//         }

//         start.elapsed() < timeout
//     }

//     pub async fn wait_for_async(&self, timeout: Duration) -> Box<dyn Future<Output = bool>> {
//         if timeout == Duration::ZERO {
//             self.wait_async().await;
//             return Box::new(async { true });
//         }

//         let start = Instant::now();
//         let mut finished = self.finished.lock().unwrap();

//         while !*finished && start.elapsed() < timeout {
//             let result = self
//                 .finished_cond
//                 .wait_timeout(finished, self.options.pause_timeout)
//                 .unwrap();
//             finished = result.0;
//             thread::sleep(self.options.pause_timeout);

//             if result.1.timed_out() || start.elapsed() >= timeout {
//                 return Box::new(async move { false });
//             }
//         }

//         Box::new(async move { start.elapsed() < timeout })
//     }
// }
