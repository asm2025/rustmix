use anyhow::Result;
use std::fs;
pub use std::path::{Path, PathBuf};

pub fn current() -> PathBuf {
    std::env::current_dir().unwrap()
}

pub fn exists<T: AsRef<Path>>(path: T) -> bool {
    let path = path.as_ref();
    path.exists() && path.is_dir()
}

pub fn create<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();

    if path.exists() {
        return Ok(());
    }

    fs::create_dir(path).map_err(Into::into)
}

pub fn ensure<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();

    if path.exists() {
        return Ok(());
    }

    fs::create_dir_all(path).map_err(Into::into)
}

pub fn delete<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(());
    }

    fs::remove_dir_all(path).map_err(Into::into)
}
