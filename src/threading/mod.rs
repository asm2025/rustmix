pub mod consumer;
pub mod injector_consumer;
pub mod parallel_consumer;
pub mod producer_consumer;

use std::{fmt, time::Duration};

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
