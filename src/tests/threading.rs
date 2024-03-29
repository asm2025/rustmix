use anyhow::Result;
use crossbeam::sync::WaitGroup;
use std::{
    collections,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use rustmix::threading::{
    consumer::*, injector_consumer::*, parallel_consumer::*, producer_consumer::*, *,
};

const THREADS: usize = 4;
const TEST_SIZE: usize = 10000;

#[derive(Debug, Clone)]
pub struct TaskHandler {
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

impl TaskDelegation<ProducerConsumer<usize>, usize> for TaskHandler {
    fn on_started(&self, _pc: &ProducerConsumer<usize>) {
        println!("Producer/Consumer started");
    }

    fn process(&self, _pc: &ProducerConsumer<usize>, item: &usize) -> Result<TaskResult> {
        self.task_count.fetch_add(1, Ordering::SeqCst);
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

    async fn process_async(
        &self,
        _td: &ProducerConsumer<usize>,
        item: &usize,
    ) -> Result<TaskResult> {
        self.task_count.fetch_add(1, Ordering::SeqCst);
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

    fn on_completed(
        &self,
        _pc: &ProducerConsumer<usize>,
        item: &usize,
        result: &TaskResult,
    ) -> bool {
        self.done_count.fetch_add(1, Ordering::SeqCst);
        println!("Result item: {}: {:?}", item, result);
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

impl TaskDelegation<Consumer<usize>, usize> for TaskHandler {
    fn on_started(&self, _pc: &Consumer<usize>) {
        println!("Consumer started");
    }

    fn process(&self, _pc: &Consumer<usize>, item: &usize) -> Result<TaskResult> {
        self.task_count.fetch_add(1, Ordering::SeqCst);
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

    async fn process_async(&self, _td: &Consumer<usize>, item: &usize) -> Result<TaskResult> {
        self.task_count.fetch_add(1, Ordering::SeqCst);
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

    fn on_completed(&self, _pc: &Consumer<usize>, item: &usize, result: &TaskResult) -> bool {
        self.done_count.fetch_add(1, Ordering::SeqCst);
        println!("Result item: {}: {:?}", item, result);
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

impl TaskDelegation<InjectorWorker<usize>, usize> for TaskHandler {
    fn on_started(&self, _pc: &InjectorWorker<usize>) {
        println!("Injector/Worker started");
    }

    fn process(&self, _pc: &InjectorWorker<usize>, item: &usize) -> Result<TaskResult> {
        self.task_count.fetch_add(1, Ordering::SeqCst);
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

    async fn process_async(&self, _td: &InjectorWorker<usize>, item: &usize) -> Result<TaskResult> {
        self.task_count.fetch_add(1, Ordering::SeqCst);
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

    fn on_completed(&self, _pc: &InjectorWorker<usize>, item: &usize, result: &TaskResult) -> bool {
        self.done_count.fetch_add(1, Ordering::SeqCst);
        println!("Result item: {}: {:?}", item, result);
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

impl TaskDelegation<Parallel<usize>, usize> for TaskHandler {
    fn on_started(&self, _pc: &Parallel<usize>) {
        println!("Parallel started");
    }

    fn process(&self, _pc: &Parallel<usize>, item: &usize) -> Result<TaskResult> {
        self.task_count.fetch_add(1, Ordering::SeqCst);
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

    async fn process_async(&self, _td: &Parallel<usize>, item: &usize) -> Result<TaskResult> {
        self.task_count.fetch_add(1, Ordering::SeqCst);
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

    fn on_completed(&self, _pc: &Parallel<usize>, item: &usize, result: &TaskResult) -> bool {
        self.done_count.fetch_add(1, Ordering::SeqCst);
        println!("Result item: {}: {:?}", item, result);
        true
    }

    fn on_finished(&self, _pc: &Parallel<usize>) {
        println!(
            "Got: {} tasks and finished {} tasks.",
            self.task_count.load(Ordering::SeqCst),
            self.done_count.load(Ordering::SeqCst)
        );
    }
}

pub async fn test_producer_consumer() -> Result<()> {
    println!("\nTesting Producer/Consumer with {} threads...", THREADS);

    let now = Instant::now();
    let handler = TaskHandler::new();
    let options = ProducerConsumerOptions::new();
    let prodcon = ProducerConsumer::<usize>::with_options(options);
    let unit = TEST_SIZE / THREADS;

    for _ in 0..THREADS {
        prodcon.start_consumer(&handler.clone());
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

pub async fn test_consumer() -> Result<()> {
    println!("\nTesting Consumer with {} threads...", THREADS);

    let now = Instant::now();
    let handler = TaskHandler::new();
    let options = ConsumerOptions::new();
    let consumer = Consumer::<usize>::with_options(options);

    for _ in 0..THREADS {
        consumer.start(&handler.clone());
    }

    for i in 1..=TEST_SIZE {
        consumer.enqueue(i);
    }

    consumer.complete();
    let _ = consumer.wait_async().await;

    println!("Elapsed time: {:?}", now.elapsed());
    Ok(())
}

pub async fn test_injector_worker() -> Result<()> {
    println!("\nTesting Injector/Worker with {} threads...", THREADS);

    let now = Instant::now();
    let handler = TaskHandler::new();
    let options = InjectorWorkerOptions::new().with_threads(THREADS);
    let injwork = InjectorWorker::<usize>::with_options(options);
    injwork.start(&handler);

    for i in 1..=TEST_SIZE {
        injwork.enqueue(i);
    }

    injwork.complete();
    let _ = injwork.wait_async().await;

    println!("Elapsed time: {:?}", now.elapsed());
    Ok(())
}

pub async fn test_parallel() -> Result<()> {
    println!("\nTesting Parallel with {} threads...", THREADS);

    let now = Instant::now();
    let handler = TaskHandler::new();
    let options = ParallelOptions::new().with_threads(THREADS);
    let parallel = Parallel::with_options(options);
    let mut collection = collections::VecDeque::<usize>::with_capacity(TEST_SIZE);

    for i in 1..=TEST_SIZE {
        collection.push_back(i);
    }

    parallel.start(collection, &handler);
    let _ = parallel.wait_async().await;

    println!("Elapsed time: {:?}", now.elapsed());
    Ok(())
}
