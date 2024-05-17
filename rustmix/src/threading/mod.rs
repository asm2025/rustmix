mod cond;
pub use self::cond::*;
mod consumer;
pub use self::consumer::*;
mod injector_consumer;
pub use self::injector_consumer::*;
mod producer_consumer;
pub use self::producer_consumer::*;
mod spinner;
pub use self::spinner::*;

use futures::Future;
use std::{fmt, pin::Pin, sync::Arc, thread};
use tokio::{
    sync::Notify,
    time::{self, Duration},
};

use crate::{
    error::{CancelledError, TimedoutError},
    Result,
};

const CAPACITY_DEF: usize = 0;
const THREADS_DEF: usize = 1;
pub const THREADS_MIN: usize = 1;
pub const THREADS_MAX: usize = 255;
const QUEUE_BEHAVIOR_DEF: QueueBehavior = QueueBehavior::FIFO;
const THRESHOLD_DEF: Duration = Duration::ZERO;
const SLEEP_AFTER_SEND_DEF: Duration = Duration::ZERO;
const PEEK_TIMEOUT_DEF: Duration = Duration::from_millis(50);
const PEEK_TIMEOUT_MIN: Duration = Duration::from_millis(10);
const PEEK_TIMEOUT_MAX: Duration = Duration::from_secs(5);
const PAUSE_TIMEOUT_DEF: Duration = Duration::from_millis(50);
const PAUSE_TIMEOUT_MIN: Duration = Duration::from_millis(10);
const PAUSE_TIMEOUT_MAX: Duration = Duration::from_secs(5);
pub const INTERVAL: u64 = 100;

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

pub trait TaskItem: Clone + Send + Sync + fmt::Debug {}
impl<T: Clone + Send + Sync + fmt::Debug> TaskItem for T {}

pub trait StaticTaskItem: TaskItem + 'static {}
impl<T: TaskItem + 'static> StaticTaskItem for T {}

pub trait TaskDelegation<TPC: AwaitableConsumer<T>, T: StaticTaskItem>: StaticTaskItem {
    fn on_started(&self);
    fn process(&self, pc: &TPC, item: &T) -> Result<TaskResult>;
    fn on_completed(&self, item: &T, result: &TaskResult) -> bool;
    fn on_cancelled(&self);
    fn on_finished(&self);
}

pub trait AwaitableConsumer<T: TaskItem>: StaticTaskItem {
    fn is_cancelled(&self) -> bool;
    fn is_finished(&self) -> bool;
}

fn wait<TPC: AwaitableConsumer<T>, T: StaticTaskItem>(
    this: &TPC,
    finished: &Arc<Mutcond>,
) -> Result<()> {
    match finished.wait_while(|| !this.is_cancelled() && !this.is_finished()) {
        Ok(_) => {
            if this.is_cancelled() {
                Err(CancelledError.into())
            } else {
                Ok(())
            }
        }
        Err(_) => Err(CancelledError.into()),
    }
}

async fn wait_async<TPC: AwaitableConsumer<T>, T: StaticTaskItem>(
    this: &TPC,
    finished: &Arc<Notify>,
) -> Result<()> {
    let mut notified = false;

    while !notified && !this.is_finished() && !this.is_cancelled() {
        thread::sleep(Duration::ZERO);
        finished.notified().await;
        notified = true;
    }

    if this.is_cancelled() {
        return Err(CancelledError.into());
    }

    Ok(())
}

fn wait_until<TPC: AwaitableConsumer<T>, T: StaticTaskItem>(
    this: &TPC,
    finished: &Arc<Mutcond>,
    cond: impl Fn(&TPC) -> bool,
) -> Result<()> {
    match finished.wait_while(|| !this.is_cancelled() && !this.is_finished() && !cond(this)) {
        Ok(_) => {
            if this.is_cancelled() {
                Err(CancelledError.into())
            } else {
                Ok(())
            }
        }
        Err(_) => Err(CancelledError.into()),
    }
}

async fn wait_until_async<
    TPC: AwaitableConsumer<T>,
    T: StaticTaskItem,
    F: Fn(&TPC) -> Pin<Box<dyn Future<Output = bool> + Send>>,
>(
    this: &TPC,
    finished: &Arc<Notify>,
    cond: F,
) -> Result<()> {
    let mut notified = false;

    while !cond(this).await && !notified && !this.is_cancelled() && !this.is_finished() {
        finished.notified().await;
        notified = true;
    }

    if this.is_cancelled() {
        return Err(CancelledError.into());
    }

    Ok(())
}

fn wait_for<TPC: AwaitableConsumer<T>, T: StaticTaskItem>(
    this: &TPC,
    timeout: Duration,
    finished: &Arc<Mutcond>,
) -> Result<()> {
    if timeout.is_zero() {
        return Err(TimedoutError.into());
    }

    match finished.wait_timeout_while(|| !this.is_cancelled() && !this.is_finished(), timeout) {
        Ok(_) => {
            if this.is_cancelled() {
                Err(CancelledError.into())
            } else {
                Ok(())
            }
        }
        Err(_) => Err(TimedoutError.into()),
    }
}

async fn wait_for_async<TPC: AwaitableConsumer<T>, T: StaticTaskItem>(
    this: &TPC,
    timeout: Duration,
    finished: &Arc<Notify>,
) -> Result<()> {
    if timeout.is_zero() {
        return Err(TimedoutError.into());
    }

    let result = time::timeout(timeout, finished.notified()).await;
    match result {
        Ok(_) => {
            if this.is_cancelled() {
                Err(CancelledError.into())
            } else {
                Ok(())
            }
        }
        Err(_) => Err(TimedoutError.into()),
    }
}

fn wait_for_until<TPC: AwaitableConsumer<T>, T: StaticTaskItem>(
    this: &TPC,
    timeout: Duration,
    finished: &Arc<Mutcond>,
    cond: impl Fn(&TPC) -> bool,
) -> Result<()> {
    if timeout.is_zero() {
        return Err(TimedoutError.into());
    }
    match finished.wait_timeout_while(
        || !this.is_cancelled() && !this.is_finished() && !cond(this),
        timeout,
    ) {
        Ok(_) => {
            if this.is_cancelled() {
                Err(CancelledError.into())
            } else {
                Ok(())
            }
        }
        Err(_) => Err(TimedoutError.into()),
    }
}

async fn wait_for_until_async<
    TPC: AwaitableConsumer<T>,
    T: StaticTaskItem,
    F: Fn(&TPC) -> Pin<Box<dyn Future<Output = bool> + Send>>,
>(
    this: &TPC,
    timeout: Duration,
    finished: &Arc<Notify>,
    cond: F,
) -> Result<()> {
    if timeout.is_zero() {
        return Err(TimedoutError.into());
    }

    let start = time::Instant::now();

    while !cond(this).await {
        if this.is_cancelled() {
            return Err(CancelledError.into());
        }

        if time::Instant::now().duration_since(start) > timeout {
            return Err(TimedoutError.into());
        }

        match time::timeout(PEEK_TIMEOUT_DEF, finished.notified()).await {
            Ok(_) => {
                if this.is_cancelled() {
                    return Err(CancelledError.into());
                }

                return Ok(());
            }
            Err(_) => {
                if time::Instant::now().duration_since(start) > timeout {
                    return Err(TimedoutError.into());
                }
            }
        }
    }

    Ok(())
}
