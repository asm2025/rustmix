use anyhow::{anyhow, Result};
use futures::Future;
use std::{fmt, sync::Arc, time::Instant};
use tokio::{
    select,
    sync::Notify,
    time::{sleep, Duration},
};

pub use self::cond::Mutcond;
pub use self::consumer::*;
pub use self::injector_consumer::*;
pub use self::parallel_consumer::*;
pub use self::producer_consumer::*;

use super::error;

mod cond;
mod consumer;
mod injector_consumer;
mod parallel_consumer;
mod producer_consumer;

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
const SELECT_TIMEOUT: Duration = Duration::from_secs(60);

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

fn wait(this: &impl AwaitableConsumer, finished: &Arc<Mutcond>) -> Result<()> {
    if this.is_cancelled() {
        return Err(anyhow!(error::CancelledError));
    }

    if this.is_finished() {
        return Ok(());
    }

    match finished.wait_while(|| !this.is_cancelled() && !this.is_finished()) {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow!(error::CancelledError)),
    }
}

async fn wait_async(this: &impl AwaitableConsumer, finished: &Arc<Notify>) -> Result<()> {
    if this.is_cancelled() {
        return Err(anyhow!(error::CancelledError));
    }

    if this.is_finished() {
        return Ok(());
    }

    let start = Instant::now();

    while !this.is_finished() && !this.is_cancelled() {
        select! {
            _ = finished.notified() => {},
            _ = sleep(PAUSE_TIMEOUT_DEF) => {},
            _ = sleep(SELECT_TIMEOUT) => {}
        }

        if start.elapsed() > SELECT_TIMEOUT {
            return Err(anyhow!(error::TimedoutError));
        }
    }

    if this.is_cancelled() {
        return Err(anyhow!(error::CancelledError));
    }

    Ok(())
}

fn wait_for(
    this: &impl AwaitableConsumer,
    timeout: Duration,
    finished: &Arc<Mutcond>,
) -> Result<bool> {
    if this.is_cancelled() {
        return Err(anyhow!(error::CancelledError));
    }

    if this.is_finished() {
        return Ok(true);
    }

    if timeout.is_zero() {
        if this.is_finished() {
            return Ok(true);
        }

        return Err(anyhow!(error::TimedoutError));
    }

    match finished.wait_timeout_while(|| !this.is_cancelled() && !this.is_finished(), timeout) {
        Ok(_) => {
            if this.is_cancelled() {
                return Err(anyhow!(error::CancelledError));
            }

            if this.is_finished() {
                return Ok(true);
            }

            Err(anyhow!(error::TimedoutError))
        }
        Err(_) => Err(anyhow!(error::TimedoutError)),
    }
}

async fn wait_for_async(
    this: &impl AwaitableConsumer,
    timeout: Duration,
    finished: &Arc<Notify>,
) -> Result<bool> {
    if this.is_cancelled() {
        return Err(anyhow!(error::CancelledError));
    }

    if this.is_finished() {
        return Ok(true);
    }

    if timeout.is_zero() {
        if this.is_finished() {
            return Ok(true);
        }

        return Err(anyhow!(error::TimedoutError));
    }

    let start = Instant::now();

    while !this.is_finished() && !this.is_cancelled() && start.elapsed() < timeout {
        select! {
            _ = finished.notified() => {},
            _ = sleep(PAUSE_TIMEOUT_DEF) => {},
            _ = sleep(timeout) => {}
        }

        if start.elapsed() > timeout {
            return Err(anyhow!(error::TimedoutError));
        }
    }

    if this.is_cancelled() {
        return Err(anyhow!(error::CancelledError));
    }

    if start.elapsed() <= timeout {
        Ok(true)
    } else {
        Err(anyhow!(error::TimedoutError))
    }
}
