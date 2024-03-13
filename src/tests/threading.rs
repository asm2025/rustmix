use std::{
    collections,
    error::Error,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use crossbeam::sync::WaitGroup;
use rustmix::threading::{
    consumer::*, injector_consumer::*, parallel_consumer::*, producer_consumer::*, *,
};

const THREADS: usize = 4;
const TEST_SIZE: usize = 10000;
const THREADS_NAME: &str = "<Uknown>";

#[derive(Debug, Clone)]
struct TaskHandler {
    pub task_count: Arc<AtomicUsize>,
    pub done_count: Arc<AtomicUsize>,
}

impl TaskHandler {
    pub fn new() -> Self {
        TaskHandler {
            task_count: Arc::new(AtomicUsize::new(0)),
            done_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl ProducerConsumerDelegation<usize> for TaskHandler {
    fn process(
        &self,
        _pc: &ProducerConsumer<usize>,
        item: &usize,
    ) -> Result<TaskResult, Box<dyn Error>> {
        let thread = thread::current();
        let thread_name = thread.name().unwrap_or(THREADS_NAME);

        self.task_count.fetch_add(1, Ordering::SeqCst);
        println!("Item: {} in thread: {}", item, thread_name);

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

    fn on_completed(
        &self,
        _pc: &ProducerConsumer<usize>,
        item: &usize,
        result: TaskResult,
    ) -> bool {
        let thread = thread::current();
        let thread_name = thread.name().unwrap_or(THREADS_NAME);
        self.done_count.fetch_add(1, Ordering::SeqCst);
        println!(
            "Result item: {}: {:?} in thread: {}",
            item, result, thread_name
        );
        true
    }

    fn on_finished(&self, _pc: &ProducerConsumer<usize>) {
        println!(
            "Got: {} tasks and finished {} tasks.",
            self.task_count.load(Ordering::SeqCst),
            self.done_count.load(Ordering::SeqCst)
        );
    }
}

impl ConsumerDelegation<usize> for TaskHandler {
    fn process(&self, _pc: &Consumer<usize>, item: &usize) -> Result<TaskResult, Box<dyn Error>> {
        let thread = thread::current();
        let thread_name = thread.name().unwrap_or(THREADS_NAME);
        self.task_count.fetch_add(1, Ordering::SeqCst);
        println!("Item: {} in thread: {}", item, thread_name);

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

    fn on_completed(&self, _pc: &Consumer<usize>, item: &usize, result: TaskResult) -> bool {
        let thread = thread::current();
        let thread_name = thread.name().unwrap_or(THREADS_NAME);
        self.done_count.fetch_add(1, Ordering::SeqCst);
        println!(
            "Result item: {}: {:?} in thread: {}",
            item, result, thread_name
        );
        true
    }

    fn on_finished(&self, _pc: &Consumer<usize>) {
        println!(
            "Got: {} tasks and finished {} tasks.",
            self.task_count.load(Ordering::SeqCst),
            self.done_count.load(Ordering::SeqCst)
        );
    }
}

impl InjectorWorkerDelegation<usize> for TaskHandler {
    fn process(
        &self,
        _pc: &InjectorWorker<usize>,
        item: &usize,
    ) -> Result<TaskResult, Box<dyn Error>> {
        let thread = thread::current();
        let thread_name = thread.name().unwrap_or(THREADS_NAME);
        self.task_count.fetch_add(1, Ordering::SeqCst);
        println!("Item: {} in thread: {}", item, thread_name);

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

    fn on_completed(&self, _pc: &InjectorWorker<usize>, item: &usize, result: TaskResult) -> bool {
        let thread = thread::current();
        let thread_name = thread.name().unwrap_or(THREADS_NAME);
        self.done_count.fetch_add(1, Ordering::SeqCst);
        println!(
            "Result item: {}: {:?} in thread: {}",
            item, result, thread_name
        );
        true
    }

    fn on_finished(&self, _pc: &InjectorWorker<usize>) {
        println!(
            "Got: {} tasks and finished {} tasks.",
            self.task_count.load(Ordering::SeqCst),
            self.done_count.load(Ordering::SeqCst)
        );
    }
}

impl ParallelDelegation<usize> for TaskHandler {
    fn process(&self, _pc: &Parallel, item: &usize) -> Result<TaskResult, Box<dyn Error>> {
        let thread = thread::current();
        let thread_name = thread.name().unwrap_or(THREADS_NAME);
        self.task_count.fetch_add(1, Ordering::SeqCst);
        println!("Item: {} in thread: {}", item, thread_name);

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

    fn on_completed(&self, _pc: &Parallel, item: &usize, result: TaskResult) -> bool {
        let thread = thread::current();
        let thread_name = thread.name().unwrap_or(THREADS_NAME);
        self.done_count.fetch_add(1, Ordering::SeqCst);
        println!(
            "Result item: {}: {:?} in thread: {}",
            item, result, thread_name
        );
        true
    }

    fn on_finished(&self, _pc: &Parallel) {
        println!(
            "Got: {} tasks and finished {} tasks.",
            self.task_count.load(Ordering::SeqCst),
            self.done_count.load(Ordering::SeqCst)
        );
    }
}

pub async fn test_producer_consumer() -> Result<(), Box<dyn Error>> {
    println!("\nTesting Producer/Consumer with {} threads...", THREADS);

    let now = std::time::Instant::now();
    let handler = TaskHandler::new();
    let options = ProducerConsumerOptions::new();
    let prodcon = ProducerConsumer::<usize>::with_options(options);
    let unit = TEST_SIZE / THREADS;

    for _ in 0..THREADS {
        let con = handler.clone();
        prodcon.start_consumer(con);
    }

    let wg = WaitGroup::new();

    for n in 0..THREADS {
        let wgc = wg.clone();
        let p = prodcon.new_producer();
        thread::spawn(move || {
            for i in 1..=unit {
                p.enqueue(i + (unit * n));
            }

            drop(wgc);
        });
    }

    wg.wait();
    prodcon.complete();
    let _ = prodcon.wait_async().await;

    println!("Elapsed time: {:?}", now.elapsed());
    Ok(())
}

pub async fn test_consumer() -> Result<(), Box<dyn Error>> {
    println!("\nTesting Consumer with {} threads...", THREADS);

    let now = std::time::Instant::now();
    let handler = TaskHandler::new();
    let options = ConsumerOptions::new();
    let consumer = Consumer::<usize>::with_options(options);

    for _ in 0..THREADS {
        let con = handler.clone();
        consumer.start(con);
    }

    for i in 1..=TEST_SIZE {
        consumer.enqueue(i);
    }

    consumer.complete();
    let _ = consumer.wait_async().await;

    println!("Elapsed time: {:?}", now.elapsed());
    Ok(())
}

pub async fn test_injector_worker() -> Result<(), Box<dyn Error>> {
    println!("\nTesting Injector/Worker with {} threads...", THREADS);

    let now = std::time::Instant::now();
    let handler = TaskHandler::new();
    let options = InjectorWorkerOptions::new().with_threads(THREADS);
    let injwork = InjectorWorker::<usize>::with_options(options);
    injwork.start(handler);

    for i in 1..=TEST_SIZE {
        injwork.enqueue(i);
    }

    injwork.complete();
    let _ = injwork.wait_async().await;

    println!("Elapsed time: {:?}", now.elapsed());
    Ok(())
}

pub async fn test_parallel() -> Result<(), Box<dyn Error>> {
    println!("\nTesting Parallel with {} threads...", THREADS);

    let now = std::time::Instant::now();
    let handler = TaskHandler::new();
    let options = ParallelOptions::new().with_threads(1);
    let parallel = Parallel::with_options(options);
    let mut collection = collections::VecDeque::<usize>::with_capacity(TEST_SIZE);

    for i in 1..=TEST_SIZE {
        collection.push_back(i);
    }

    parallel.start(collection, handler);
    parallel.complete();
    let _ = parallel.wait_async().await;

    println!("Elapsed time: {:?}", now.elapsed());
    Ok(())
}
