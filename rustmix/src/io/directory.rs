use std::fs;
pub use std::path::{Path, PathBuf};

use crate::Result;

pub fn current() -> PathBuf {
    std::env::current_dir().unwrap()
}

pub fn exists<T: AsRef<Path>>(path: T) -> bool {
    path.as_ref().is_dir()
}

pub fn create<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();

    if path.is_dir() {
        return Ok(());
    }

    fs::create_dir(path).map_err(Into::into)
}

pub fn ensure<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();

    if path.is_dir() {
        return Ok(());
    }

    fs::create_dir_all(path).map_err(Into::into)
}

pub fn is_empty<T: AsRef<Path>>(path: T) -> bool {
    fs::read_dir(path.as_ref()).map_or(false, |mut i| i.next().is_none())
}
