use colored::Colorize;
use std::{
    error::Error,
    sync::{atomic::AtomicUsize, Arc},
    thread,
    time::Duration,
};

use rustmix::threading::prodcon::*;

#[derive(Debug, Clone)]
struct ProCon {
    pub task_count: Arc<AtomicUsize>,
    pub done_count: Arc<AtomicUsize>,
}

impl ProCon {
    pub fn new() -> Self {
        ProCon {
            task_count: Arc::new(AtomicUsize::new(0)),
            done_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl TaskDelegate<usize> for ProCon {
    fn on_task(
        &self,
        _pc: &ProducerConsumer<usize>,
        item: usize,
    ) -> Result<TaskResult, Box<dyn Error>> {
        let current_thread = thread::current();
        self.task_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        println!(
            "Item: {} in thread: {}",
            item,
            current_thread.name().unwrap()
        );

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

    fn on_result(&self, _pc: &ProducerConsumer<usize>, item: usize, result: TaskResult) -> bool {
        let current_thread = thread::current();
        self.done_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        println!(
            "Result item: {}: {:?} in thread: {}",
            item,
            result,
            current_thread.name().unwrap()
        );
        true
    }

    fn on_finished(&self, _pc: &ProducerConsumer<usize>) {
        println!(
            "Got: {} tasks and finished {} tasks.",
            self.task_count.load(std::sync::atomic::Ordering::Relaxed),
            self.done_count.load(std::sync::atomic::Ordering::Relaxed)
        );
    }
}

pub async fn test_prodcon(threads: usize) -> Result<(), Box<dyn Error>> {
    let th = if threads > 0 { threads } else { 1 };
    println!(
        "\n{} with {} threads...",
        "Testing Producer/Consumer".magenta(),
        th
    );

    let consumer = ProCon::new();
    let options = ProducerConsumerOptions::new();
    let pc = ProducerConsumer::<usize>::with_options(options);
    pc.start_producer(consumer.clone());
    thread::sleep(Duration::ZERO);

    for _ in 0..th {
        let con = consumer.clone();
        pc.start_consumer(con);
        thread::sleep(Duration::ZERO);
    }

    for i in 1..=100 {
        pc.enqueue(i);
    }

    pc.complete();
    thread::sleep(Duration::ZERO);
    let _ = pc.wait_async().await;

    Ok(())
}
