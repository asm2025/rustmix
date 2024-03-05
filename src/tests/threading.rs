use colored::Colorize;
use std::{error::Error, thread, time::Duration};

use rustmix::threading::prodcon::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Consumer;

impl TaskDelegate<usize> for Consumer {
    fn on_task(
        &self,
        _pc: &ProducerConsumer<usize>,
        item: usize,
    ) -> Result<TaskResult, Box<dyn Error>> {
        let current_thread = thread::current();
        println!(
            "Item: {} in thread: {}",
            item,
            current_thread.name().unwrap()
        );

        if item % 5 == 0 {
            return Ok(TaskResult::Error(
                "Multiples of 5 are not allowed".to_string(),
            ));
        } else if item % 3 == 0 {
            return Ok(TaskResult::TimedOut);
        }

        Ok(TaskResult::Success)
    }

    fn on_result(&self, _pc: &ProducerConsumer<usize>, item: usize, result: TaskResult) -> bool {
        let current_thread = thread::current();
        println!(
            "Recieved result: {:?} item: {} in thread: {}",
            result,
            item,
            current_thread.name().unwrap()
        );
        true
    }
}

pub async fn test_prodcon(threads: usize) -> Result<(), Box<dyn Error>> {
    let th = if threads > 0 { threads } else { 1 };
    println!(
        "\n{} with {} threads...",
        "Testing Producer/Consumer".magenta(),
        th
    );

    let consumer = Consumer;
    let options = ProducerConsumerOptions::new();
    let pc = ProducerConsumer::<usize>::with_options(options);
    pc.start_producer();

    for _ in 0..th {
        let con = consumer.clone();
        pc.start_consumer(con);
    }

    for i in 0..=100 {
        pc.enqueue(i);
        thread::sleep(Duration::from_millis(10));
    }

    pc.complete();
    let _ = pc.wait_async().await;

    Ok(())
}
