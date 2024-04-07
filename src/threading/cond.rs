use std::{
    sync::{Arc, Condvar, LockResult, Mutex, MutexGuard},
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct Mutcond {
    pair: Arc<(Mutex<bool>, Condvar)>,
}

impl Mutcond {
    pub fn new() -> Self {
        Self {
            pair: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    pub fn is_signaled(&self) -> LockResult<MutexGuard<bool>> {
        let (lock, _) = &*self.pair;
        lock.lock()
    }

    pub fn notify_one(&self) {
        let (lock, cvar) = &*self.pair;
        let mut guard = lock.lock().unwrap();
        *guard = true;
        cvar.notify_one();
    }

    pub fn notify_all(&self) {
        let (lock, cvar) = &*self.pair;
        let mut guard = lock.lock().unwrap();
        *guard = true;
        cvar.notify_all();
    }

    pub fn wait(&self) -> LockResult<()> {
        let (lock, cvar) = &*self.pair;
        let mut guard = lock.lock().unwrap();

        while !*guard {
            guard = cvar.wait(guard).unwrap();
        }

        *guard = false;
        Ok(())
    }

    pub fn wait_timeout(&self, timeout: Duration) -> LockResult<bool> {
        let (lock, cvar) = &*self.pair;
        let mut guard = lock.lock().unwrap();
        let (new_guard, result) = cvar.wait_timeout(guard, timeout).unwrap();
        guard = new_guard;

        if result.timed_out() {
            return Ok(false);
        }

        *guard = false;
        Ok(true)
    }

    pub fn wait_timeout_ms(&self, timeout: u64) -> LockResult<bool> {
        self.wait_timeout(Duration::from_millis(timeout))
    }

    pub fn wait_while(&self, condition: impl Fn() -> bool) -> LockResult<()> {
        let (lock, cvar) = &*self.pair;
        let mut guard = lock.lock().unwrap();

        while condition() {
            guard = cvar.wait(guard).unwrap();
        }

        Ok(())
    }

    pub fn wait_timeout_while(
        &self,
        condition: impl Fn() -> bool,
        timeout: Duration,
    ) -> LockResult<bool> {
        let (lock, cvar) = &*self.pair;
        let mut guard = lock.lock().unwrap();
        let start = Instant::now();

        while condition() {
            let remaining = timeout.checked_sub(start.elapsed());
            match remaining {
                Some(time) => {
                    let result = cvar.wait_timeout(guard, time).unwrap();
                    guard = result.0;
                    if result.1.timed_out() {
                        return Ok(false);
                    }
                }
                None => return Ok(false),
            }
        }

        Ok(true)
    }
}
