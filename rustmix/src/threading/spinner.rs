pub use indicatif::*;
use std::{
    borrow::Cow,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use super::INTERVAL;
use crate::{error::InvalidOperationError, Result};

const ERR_FINISHED: &str = "Spinner is already finished. Try resseting it.";

#[derive(Clone)]
pub struct SpinnerOptions {
    pub prefix: Option<String>,
    pub message: Option<String>,
    pub tab_width: Option<usize>,
    pub position: Option<u64>,
    pub style: Option<ProgressStyle>,
    pub steady_ticks: Option<u64>,
}

impl Default for SpinnerOptions {
    fn default() -> Self {
        Self {
            prefix: Default::default(),
            message: Some("Initializing...".to_string()),
            tab_width: Default::default(),
            position: Default::default(),
            style: Some(
                ProgressStyle::default_spinner()
                    .tick_chars("⣾⣽⣻⢿⡿⣟⣯⣷")
                    .template("{spinner:.green} {prefix}{msg}")
                    .unwrap(),
            ),
            steady_ticks: Some(INTERVAL),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spinner {
    pb: ProgressBar,
    is_finished: Arc<AtomicBool>,
}

impl Spinner {
    pub fn new() -> Self {
        let pb = ProgressBar::new_spinner();
        Self {
            pb: Self::setup(pb, SpinnerOptions::default()),
            is_finished: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_prefix(prefix: String) -> Self {
        let pb = ProgressBar::new_spinner();
        let mut options = SpinnerOptions::default();
        options.prefix = Some(prefix);
        Self {
            pb: Self::setup(pb, options),
            is_finished: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_style(style: ProgressStyle) -> Self {
        let pb = ProgressBar::new_spinner();
        let mut options = SpinnerOptions::default();
        options.style = Some(style);
        Self {
            pb: Self::setup(pb, options),
            is_finished: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_elapsed(elapsed: Duration) -> Self {
        let pb = ProgressBar::new_spinner().with_elapsed(elapsed);
        Self {
            pb: Self::setup(pb, SpinnerOptions::default()),
            is_finished: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_finish(finish: ProgressFinish) -> Self {
        let pb = ProgressBar::new_spinner().with_finish(finish);
        Self {
            pb: Self::setup(pb, SpinnerOptions::default()),
            is_finished: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_options(options: SpinnerOptions) -> Self {
        let pb = ProgressBar::new_spinner();
        Self {
            pb: Self::setup(pb, options),
            is_finished: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with(
        elapsed: Option<Duration>,
        finish: Option<ProgressFinish>,
        options: SpinnerOptions,
    ) -> Self {
        let pb = if elapsed.is_some() && finish.is_some() {
            ProgressBar::new_spinner()
                .with_elapsed(elapsed.unwrap())
                .with_finish(finish.unwrap())
        } else if elapsed.is_some() {
            ProgressBar::new_spinner().with_elapsed(elapsed.unwrap())
        } else if finish.is_some() {
            ProgressBar::new_spinner().with_finish(finish.unwrap())
        } else {
            ProgressBar::new_spinner()
        };
        Self {
            pb: Self::setup(pb, options),
            is_finished: Arc::new(AtomicBool::new(false)),
        }
    }

    fn setup(mut pd: ProgressBar, options: SpinnerOptions) -> ProgressBar {
        if let Some(prefix) = options.prefix {
            pd.set_prefix(prefix);
        }

        if let Some(message) = options.message {
            pd.set_message(message);
        }

        if let Some(tab_width) = options.tab_width {
            pd.set_tab_width(tab_width);
        }

        if let Some(position) = options.position {
            pd.set_position(position);
        }

        if let Some(style) = options.style {
            pd.set_style(style);
        }

        if let Some(interval) = options.steady_ticks {
            if interval > 0 {
                pd.enable_steady_tick(Duration::from_millis(interval));
            }
        }

        pd.tick();
        pd
    }

    pub fn spin(&self) -> Result<()> {
        if self.is_finished() {
            return Err(InvalidOperationError(ERR_FINISHED.to_string()).into());
        }

        let is_finished = self.is_finished.clone();
        let handle = thread::spawn(move || {
            while !is_finished.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100));
            }
        });
        handle.join().unwrap();
        Ok(())
    }

    pub fn tick(&self) {
        self.pb.tick();
    }

    pub fn run<T: Send + 'static, F: FnOnce() -> T + Send + 'static>(
        &self,
        process: F,
    ) -> Result<T> {
        let is_finished = self.is_finished.clone();
        let handle = thread::spawn(move || {
            let result = process();
            is_finished.store(true, Ordering::Relaxed);
            result
        });

        while !self.is_finished.load(Ordering::SeqCst) {
            self.pb.tick();
            thread::sleep(Duration::from_millis(INTERVAL));
        }

        let result = handle.join().unwrap();
        Ok(result)
    }

    pub fn suspend<F: FnOnce() -> R, R>(&self, f: F) -> R {
        self.pb.suspend(f)
    }

    pub fn set_steady_tick(&self, interval: u64) {
        if interval > 0 {
            self.pb.enable_steady_tick(Duration::from_millis(interval));
        } else {
            self.pb.disable_steady_tick();
        }
    }

    pub fn message(&self) -> String {
        self.pb.message().to_string()
    }

    pub fn set_message(&self, message: impl Into<Cow<'static, str>>) {
        self.pb.set_message(message);
    }

    pub fn prefix(&self) -> String {
        self.pb.prefix().to_string()
    }

    pub fn set_prefix(&self, prefix: impl Into<Cow<'static, str>>) {
        self.pb.set_prefix(prefix);
    }

    pub fn duration(&self) -> Duration {
        self.pb.duration()
    }

    pub fn elapsed(&self) -> Duration {
        self.pb.elapsed()
    }

    pub fn eta(&self) -> Duration {
        self.pb.eta()
    }

    pub fn is_finished(&self) -> bool {
        self.is_finished.load(Ordering::Relaxed)
    }

    pub fn finish(&self) -> Result<()> {
        if self.is_finished() {
            return Err(InvalidOperationError(ERR_FINISHED.to_string()).into());
        }

        self.is_finished.store(true, Ordering::Relaxed);
        self.pb.finish();
        Ok(())
    }

    pub fn finish_with_message(&self, message: impl Into<Cow<'static, str>>) -> Result<()> {
        if self.is_finished() {
            return Err(InvalidOperationError(ERR_FINISHED.to_string()).into());
        }

        self.is_finished.store(true, Ordering::Relaxed);
        self.pb.finish_with_message(message);
        Ok(())
    }

    pub fn finish_using_style(&self) -> Result<()> {
        if self.is_finished() {
            return Err(InvalidOperationError(ERR_FINISHED.to_string()).into());
        }

        self.is_finished.store(true, Ordering::Relaxed);
        self.pb.finish_using_style();
        Ok(())
    }

    pub fn finish_and_clear(&self) -> Result<()> {
        if self.is_finished() {
            return Err(InvalidOperationError(ERR_FINISHED.to_string()).into());
        }

        self.is_finished.store(true, Ordering::Relaxed);
        self.pb.finish_and_clear();
        Ok(())
    }

    pub fn abandon(&self) -> Result<()> {
        if self.is_finished() {
            return Err(InvalidOperationError(ERR_FINISHED.to_string()).into());
        }

        self.is_finished.store(true, Ordering::Relaxed);
        self.pb.abandon();
        Ok(())
    }

    pub fn abandon_with_message(&self, message: String) -> Result<()> {
        if self.is_finished() {
            return Err(InvalidOperationError(ERR_FINISHED.to_string()).into());
        }

        self.is_finished.store(true, Ordering::Relaxed);
        self.pb.abandon_with_message(message);
        Ok(())
    }

    pub fn reset_elapsed(&self) {
        self.pb.reset_elapsed();
    }

    pub fn reset_eta(&self) {
        self.pb.reset_eta();
    }

    pub fn reset(&self) -> Result<()> {
        if !self.is_finished() {
            self.pb.finish_and_clear();
        }

        self.pb.reset();
        self.is_finished.store(false, Ordering::Relaxed);
        Ok(())
    }
}
