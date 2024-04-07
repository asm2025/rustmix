use anyhow::{Error, Result};
use futures::executor::block_on;
use std::{fmt, future::Future, sync::Arc};
use tokio::{
    select,
    sync::Notify,
    time::{sleep as tokio_sleep, timeout as tokio_timeout, Duration, Instant},
};

use super::error;

pub mod consumer;
pub mod injector_consumer;
pub mod parallel_consumer;
pub mod producer_consumer;

const CAPACITY_DEF: usize = 0;
const THREADS_DEF: usize = 1;
const THREADS_MIN: usize = 1;
const THREADS_MAX: usize = 255;
const QUEUE_BEHAVIOR_DEF: QueueBehavior = QueueBehavior::FIFO;
const THRESHOLD_DEF: Duration = Duration::ZERO;
const SLEEP_AFTER_SEND_DEF: Duration = Duration::ZERO;
const PEEK_TIMEOUT_DEF: Duration = Duration::from_millis(50);
const PEEK_TIMEOUT_MIN: Duration = Duration::from_millis(10);
const PEEK_TIMEOUT_MAX: Duration = Duration::from_secs(5);
const PAUSE_TIMEOUT_DEF: Duration = Duration::from_millis(50);
const PAUSE_TIMEOUT_MIN: Duration = Duration::from_millis(10);
const PAUSE_TIMEOUT_MAX: Duration = Duration::from_secs(5);

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskResult {
    #[default]
    None,
    Cancelled,
    TimedOut,
    Error(String),
    Success,
}

impl fmt::Display for TaskResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskResult::Cancelled => write!(f, "Cancelled"),
            TaskResult::TimedOut => write!(f, "Timedout"),
            TaskResult::Error(e) => write!(f, "Error: {}", e),
            TaskResult::Success => write!(f, "Success"),
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QueueBehavior {
    #[default]
    FIFO,
    LIFO,
}

impl fmt::Display for QueueBehavior {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QueueBehavior::FIFO => write!(f, "FIFO"),
            QueueBehavior::LIFO => write!(f, "LIFO"),
        }
    }
}

pub trait TaskDelegationBase<TD: Send + Clone + 'static, T: Send + Clone + 'static> {
    fn on_started(&self, td: &TD);
    fn on_completed(&self, td: &TD, item: &T, result: &TaskResult) -> bool;
    fn on_cancelled(&self, td: &TD);
    fn on_finished(&self, td: &TD);
}

pub trait TaskDelegation<TD: Send + Clone + 'static, T: Send + Clone + 'static>:
    TaskDelegationBase<TD, T>
{
    fn process(&self, td: &TD, item: &T) -> Result<TaskResult>;
}

pub trait AsyncTaskDelegation<TD: Send + Clone + 'static, T: Send + Clone + 'static>:
    TaskDelegationBase<TD, T>
{
    fn process(&self, td: &TD, item: &T) -> impl Future<Output = Result<TaskResult>> + Send;
}

pub trait AwaitableConsumer {
    fn is_cancelled(&self) -> bool;
    fn is_finished(&self) -> bool;
}

fn wait(
    this: &impl AwaitableConsumer,
    pause_timeout: Duration,
    finished: &Arc<Notify>,
) -> Result<()> {
    if this.is_cancelled() {
        return Err(Error::new(error::CancelledError));
    }

    while !this.is_finished() {
        let result = block_on(tokio_timeout(pause_timeout, finished.notified()));
        if result.is_err() {
            break;
        }
    }

    if this.is_cancelled() {
        return Err(Error::new(error::CancelledError));
    }

    Ok(())
}

async fn wait_async(
    this: &impl AwaitableConsumer,
    pause_timeout: Duration,
    finished: &Arc<Notify>,
) -> Result<()> {
    if this.is_cancelled() {
        return Err(Error::new(error::CancelledError));
    }

    while !this.is_finished() {
        select! {
            _ = finished.notified() => {},
            _ = tokio_sleep(pause_timeout) => {}
        }
    }

    if this.is_cancelled() {
        return Err(Error::new(error::CancelledError));
    }

    Ok(())
}

fn wait_for(
    this: &impl AwaitableConsumer,
    timeout: Duration,
    pause_timeout: Duration,
    finished: &Arc<Notify>,
) -> Result<bool> {
    if this.is_cancelled() {
        return Err(Error::new(error::CancelledError));
    }

    if timeout.is_zero() {
        if this.is_finished() {
            return Ok(true);
        }

        return Err(Error::new(error::TimedoutError));
    }

    let start = Instant::now();

    while !this.is_finished() && start.elapsed() < timeout {
        let wait_timeout = timeout - start.elapsed();
        let pause_timeout = pause_timeout.min(wait_timeout);
        let result = block_on(tokio_timeout(pause_timeout, finished.notified()));
        if result.is_err() {
            break;
        }
    }

    if this.is_cancelled() {
        return Err(Error::new(error::CancelledError));
    }

    if start.elapsed() <= timeout {
        Ok(true)
    } else {
        Err(Error::new(error::TimedoutError))
    }
}

async fn wait_for_async(
    this: &impl AwaitableConsumer,
    timeout: Duration,
    pause_timeout: Duration,
    finished: &Arc<Notify>,
) -> Result<bool> {
    if this.is_cancelled() {
        return Err(Error::new(error::CancelledError));
    }

    if timeout.is_zero() {
        if this.is_finished() {
            return Ok(true);
        }

        return Err(Error::new(error::TimedoutError));
    }

    let start = Instant::now();

    while !this.is_finished() && start.elapsed() < timeout {
        let wait_timeout = timeout - start.elapsed();
        let pause_timeout = pause_timeout.min(wait_timeout);

        select! {
            _ = finished.notified() => {},
            _ = tokio_sleep(pause_timeout) => {}
        }
    }

    if this.is_cancelled() {
        return Err(Error::new(error::CancelledError));
    }

    if start.elapsed() <= timeout {
        Ok(true)
    } else {
        Err(Error::new(error::TimedoutError))
    }
}
