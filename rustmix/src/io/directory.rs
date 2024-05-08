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

pub fn clear<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();

    if !path.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}

pub fn delete<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();

    if !path.is_dir() {
        return Ok(());
    }

    fs::remove_dir_all(path).map_err(Into::into)
}
