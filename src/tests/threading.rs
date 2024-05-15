use rustmix::{threading::*, Result};
use std::{
    collections,
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Instant,
};
use tokio::time::Duration;

const THREADS: usize = 4;
const TEST_SIZE: usize = 10000;

#[derive(Debug, Clone)]
pub struct TaskHandler {
    pub tasks: Arc<AtomicUsize>,
    pub done: Arc<AtomicUsize>,
}

impl TaskHandler {
    pub fn new() -> Self {
        TaskHandler {
            tasks: Arc::new(AtomicUsize::new(0)),
            done: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn tasks(&self) -> usize {
        self.tasks.load(Ordering::SeqCst)
    }

    pub fn done(&self) -> usize {
        self.done.load(Ordering::SeqCst)
    }
}

impl TaskDelegation<Consumer<usize>, usize> for TaskHandler {
    fn on_started(&self) {
        println!("Consumer started");
    }

    fn process(&self, _pc: &Consumer<usize>, item: &usize) -> Result<TaskResult> {
        self.tasks.fetch_add(1, Ordering::SeqCst);
        println!("Item: {}", item);

        if item % 5 == 0 {
            return Ok(TaskResult::Error(format!(
                "Item {}. Multiples of 5 are not allowed",
                item
            )));
        } else if item % 3 == 0 {
            return Ok(TaskResult::TimedOut);
        }

        Ok(TaskResult::Success)
    }

    fn on_completed(&self, item: &usize, result: &TaskResult) -> bool {
        self.done.fetch_add(1, Ordering::SeqCst);
        println!("Result item: {}: {:?}", item, result);
        true
    }

    fn on_cancelled(&self) {
        println!(
            "Cancelled. Got: {} tasks and finished {} tasks.",
            self.tasks(),
            self.done()
        );
    }

    fn on_finished(&self) {
        println!(
            "Finished. Got: {} tasks and finished {} tasks.",
            self.tasks(),
            self.done()
        );
    }
}

pub async fn test_consumer(cancel_after: Duration) -> Result<()> {
    println!("\nTesting Consumer with {} threads...", THREADS);

    let now = Instant::now();
    let handler = TaskHandler::new();
    let options = ConsumerOptions::new();
    let consumer = Consumer::<usize>::with_options(options);

    for _ in 0..THREADS {
        consumer.start(&handler.clone())?;
    }

    for i in 1..=TEST_SIZE {
        if let Err(e) = consumer.enqueue(i) {
            println!("Enqueue error: {:?}", e);
            break;
        }
    }

    consumer.complete();

    if !cancel_after.is_zero() {
        let ptr = consumer.clone();
        thread::spawn(move || {
            thread::sleep(cancel_after);

            if ptr.is_finished() {
                return;
            }

            ptr.cancel();
        });
    }

    match consumer.wait_async().await {
        Ok(_) => println!("Consumer finished"),
        Err(e) => println!("Consumer error: {:?}", e),
    }
    println!("Elapsed time: {:?}", now.elapsed());
    Ok(())
}

// impl TaskDelegation<usize> for ProducerConsumer<usize, TaskDelegation<usize>, TaskHandler> {
//     type State = TaskHandler;

//     fn on_started(&self, _pc: &ProducerConsumer<usize>) {
//         println!("Producer/Consumer started");
//     }

//     fn process(&self, item: &usize) -> Result<TaskResult> {
//         {
//             let state = self.state.lock().unwrap();
//             state.tasks.fetch_add(1, Ordering::SeqCst);
//         }
//         println!("Item: {}", item);

//         if item % 5 == 0 {
//             return Ok(TaskResult::Error(format!(
//                 "Item {}. Multiples of 5 are not allowed",
//                 item
//             )));
//         } else if item % 3 == 0 {
//             return Ok(TaskResult::TimedOut);
//         }

//         Ok(TaskResult::Success)
//     }

//     async fn process_async(&self, item: &usize) -> Result<TaskResult> {
//         {
//             let state = self.state.lock().unwrap();
//             state.tasks.fetch_add(1, Ordering::SeqCst);
//         }
//         println!("Item: {}", item);

//         if item % 5 == 0 {
//             return Ok(TaskResult::Error(format!(
//                 "Item {}. Multiples of 5 are not allowed",
//                 item
//             )));
//         } else if item % 3 == 0 {
//             return Ok(TaskResult::TimedOut);
//         }

//         Ok(TaskResult::Success)
//     }

//     fn on_completed(
//         &self,
//         _pc: &ProducerConsumer<usize>,
//         item: &usize,
//         result: &TaskResult,
//     ) -> bool {
//         self.done.fetch_add(1, Ordering::SeqCst);
//         println!("Result item: {}: {:?}", item, result);
//         true
//     }

//     fn on_cancelled(&self, _td: &ProducerConsumer<usize>) {
//         println!(
//             "Processing tasks was cancelled. Got: {} tasks and finished {} tasks.",
//             self.tasks(),
//             self.done()
//         );
//     }

//     fn on_finished(&self, _pc: &ProducerConsumer<usize>) {
//         println!(
//             "Got: {} tasks and finished {} tasks.",
//             self.tasks(),
//             self.done()
//         );
//     }
// }

// impl TaskDelegation<usize> for InjectorWorker<usize, TaskDelegation<usize>, TaskHandler> {
//     type State = TaskHandler;

//     fn on_started(&self, _pc: &ProducerConsumer<usize>) {
//         println!("Injector/Worker started");
//     }

//     fn process(&self, item: &usize) -> Result<TaskResult> {
//         {
//             let state = self.state.lock().unwrap();
//             state.tasks.fetch_add(1, Ordering::SeqCst);
//         }
//         println!("Item: {}", item);

//         if item % 5 == 0 {
//             return Ok(TaskResult::Error(format!(
//                 "Item {}. Multiples of 5 are not allowed",
//                 item
//             )));
//         } else if item % 3 == 0 {
//             return Ok(TaskResult::TimedOut);
//         }

//         Ok(TaskResult::Success)
//     }

//     async fn process_async(&self, item: &usize) -> Result<TaskResult> {
//         self.tasks.fetch_add(1, Ordering::SeqCst);
//         println!("Item: {}", item);

//         if item % 5 == 0 {
//             return Ok(TaskResult::Error(format!(
//                 "Item {}. Multiples of 5 are not allowed",
//                 item
//             )));
//         } else if item % 3 == 0 {
//             return Ok(TaskResult::TimedOut);
//         }

//         Ok(TaskResult::Success)
//     }

//     fn on_completed(&self, _pc: &InjectorWorker<usize>, item: &usize, result: &TaskResult) -> bool {
//         self.done.fetch_add(1, Ordering::SeqCst);
//         println!("Result item: {}: {:?}", item, result);
//         true
//     }

//     fn on_cancelled(&self, _td: &InjectorWorker<usize>) {
//         println!(
//             "Processing tasks was cancelled. Got: {} tasks and finished {} tasks.",
//             self.tasks(),
//             self.done()
//         );
//     }

//     fn on_finished(&self, _pc: &InjectorWorker<usize>) {
//         println!(
//             "Got: {} tasks and finished {} tasks.",
//             self.tasks(),
//             self.done()
//         );
//     }
// }

// impl TaskDelegation<usize> for Parallel<usize, TaskDelegation<usize>, TaskHandler> {
//     type State = TaskHandler;

//     fn on_started(&self, _pc: &Parallel<usize>) {
//         println!("Parallel started");
//     }

//     fn process(&self, _pc: &Parallel<usize>, item: &usize) -> Result<TaskResult> {
//         self.tasks.fetch_add(1, Ordering::SeqCst);
//         println!("Item: {}", item);

//         if item % 5 == 0 {
//             return Ok(TaskResult::Error(format!(
//                 "Item {}. Multiples of 5 are not allowed",
//                 item
//             )));
//         } else if item % 3 == 0 {
//             return Ok(TaskResult::TimedOut);
//         }

//         Ok(TaskResult::Success)
//     }

//     async fn process_async(&self, _pc: &Parallel<usize>, item: &usize) -> Result<TaskResult> {
//         self.tasks.fetch_add(1, Ordering::SeqCst);
//         println!("Item: {}", item);

//         if item % 5 == 0 {
//             return Ok(TaskResult::Error(format!(
//                 "Item {}. Multiples of 5 are not allowed",
//                 item
//             )));
//         } else if item % 3 == 0 {
//             return Ok(TaskResult::TimedOut);
//         }

//         Ok(TaskResult::Success)
//     }

//     fn on_completed(&self, _pc: &Parallel<usize>, item: &usize, result: &TaskResult) -> bool {
//         self.done.fetch_add(1, Ordering::SeqCst);
//         println!("Result item: {}: {:?}", item, result);
//         true
//     }

//     fn on_cancelled(&self, _td: &Parallel<usize>) {
//         println!(
//             "Processing tasks was cancelled. Got: {} tasks and finished {} tasks.",
//             self.tasks(),
//             self.done()
//         );
//     }

//     fn on_finished(&self, _pc: &Parallel<usize>) {
//         println!(
//             "Got: {} tasks and finished {} tasks.",
//             self.tasks(),
//             self.done()
//         );
//     }
// }

// pub async fn test_producer_consumer(cancel_after: Duration) -> Result<()> {
//     println!("\nTesting Producer/Consumer with {} threads...", THREADS);

//     let now = Instant::now();
//     let handler = TaskHandler::new();
//     let options = ProducerConsumerOptions::new().with_threads(THREADS);
//     let prodcon = ProducerConsumer::<usize>::with_options(options);
//     let unit = TEST_SIZE / THREADS;
//     let mut handles = vec![];
//     prodcon.start(&handler.clone());

//     for n in 0..THREADS {
//         let pc = prodcon.clone();
//         let p = prodcon.new_producer();
//         let h = handler.clone();
//         let handle = thread::spawn(move || {
//             for i in 1..=unit {
//                 if pc.is_cancelled() {
//                     break;
//                 }
//                 if let Err(e) = p.enqueue(i + unit * n) {
//                     println!("Enqueue error: {:?}", e);
//                     break;
//                 }
//             }

//             drop(pc);
//             drop(h);
//         });
//         handles.push(handle);
//     }

//     if !cancel_after.is_zero() {
//         let ptr = prodcon.clone();
//         thread::spawn(move || {
//             thread::sleep(cancel_after);

//             if ptr.is_finished() {
//                 return;
//             }

//             ptr.cancel();
//         });
//     }

//     for handle in handles {
//         if prodcon.is_cancelled() {
//             break;
//         }
//         handle.join().unwrap();
//     }

//     prodcon.complete();
//     match prodcon.wait_async().await {
//         Ok(_) => println!("Producer/Consumer finished"),
//         Err(e) => println!("Producer/Consumer error: {:?}", e),
//     }
//     println!("Elapsed time: {:?}", now.elapsed());
//     Ok(())
// }

// pub async fn test_injector_worker(cancel_after: Duration) -> Result<()> {
//     println!("\nTesting Injector/Worker with {} threads...", THREADS);

//     let now = Instant::now();
//     let handler = TaskHandler::new();
//     let options = InjectorWorkerOptions::new().with_threads(THREADS);
//     let injwork = InjectorWorker::<usize>::with_options(options);
//     injwork.start(&handler);

//     for i in 1..=TEST_SIZE {
//         if let Err(e) = injwork.enqueue(i) {
//             println!("Enqueue error: {:?}", e);
//             break;
//         }
//     }

//     injwork.complete();

//     if !cancel_after.is_zero() {
//         let ptr = injwork.clone();
//         thread::spawn(move || {
//             thread::sleep(cancel_after);

//             if ptr.is_finished() {
//                 return;
//             }

//             ptr.cancel();
//         });
//     }

//     match injwork.wait_async().await {
//         Ok(_) => println!("Injector/Worker finished"),
//         Err(e) => println!("Injector/Worker error: {:?}", e),
//     }
//     println!("Elapsed time: {:?}", now.elapsed());
//     Ok(())
// }

// pub async fn test_parallel(cancel_after: Duration) -> Result<()> {
//     println!("\nTesting Parallel with {} threads...", THREADS);

//     let now = Instant::now();
//     let handler = TaskHandler::new();
//     let options = ParallelOptions::new().with_threads(THREADS);
//     let parallel = Parallel::with_options(options);
//     let mut collection = collections::VecDeque::<usize>::with_capacity(TEST_SIZE);

//     for i in 1..=TEST_SIZE {
//         collection.push_back(i);
//     }

//     parallel.start(collection, &handler);

//     if !cancel_after.is_zero() {
//         let ptr = parallel.clone();
//         thread::spawn(move || {
//             thread::sleep(cancel_after);

//             if ptr.is_finished() {
//                 return;
//             }

//             ptr.cancel();
//         });
//     }

//     match parallel.wait_async().await {
//         Ok(_) => println!("Parallel finished"),
//         Err(e) => println!("Parallel error: {:?}", e),
//     }
//     println!("Elapsed time: {:?}", now.elapsed());
//     Ok(())
// }
