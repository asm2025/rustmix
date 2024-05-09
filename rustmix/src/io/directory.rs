use std::fs::{self, DirEntry};
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

pub fn list<T: AsRef<Path>>(path: T) -> Result<impl Iterator<Item = String>> {
    let path = path.as_ref();

    if !path.is_dir() {
        return Ok(Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _>>);
    }

    let read_dir = fs::read_dir(path)?;
    let iter = read_dir.filter_map(|e| match e {
        Ok(entry) => Some(entry.file_name().to_string_lossy().to_string()),
        Err(_) => None,
    });
    Ok(Box::new(iter) as Box<dyn Iterator<Item = _>>)
}

pub fn list_filtered<T: AsRef<Path>, F: Fn(&DirEntry) -> bool + 'static>(
    path: T,
    filter: F,
) -> Result<impl Iterator<Item = String>> {
    let path = path.as_ref();

    if !path.is_dir() {
        return Ok(Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _>>);
    }

    let read_dir = fs::read_dir(path)?;
    let iter = read_dir.filter_map(move |e| match e {
        Ok(entry) => {
            if filter(&entry) {
                Some(entry.file_name().to_string_lossy().to_string())
            } else {
                None
            }
        }
        Err(_) => None,
    });
    Ok(Box::new(iter) as Box<dyn Iterator<Item = _>>)
}

pub fn rename<T: AsRef<Path>, U: AsRef<Path>>(from: T, to: U) -> Result<()> {
    fs::rename(from, to).map_err(Into::into)
}

pub fn copy<T: AsRef<Path>, U: AsRef<Path>>(from: T, to: U) -> Result<()> {
    fs::copy(from, to)?;
    Ok(())
}

pub fn move_to<T: AsRef<Path>, U: AsRef<Path>>(from: T, to: U) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    if from.is_dir() {
        fs::rename(from, to).map_err(Into::into)
    } else {
        fs::copy(from, to)?;
        fs::remove_file(from).map_err(Into::into)
    }
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
