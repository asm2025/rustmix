pub use dirs::*;
use glob::glob;
use std::{fs, rc::Rc};
pub use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

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

pub fn list_match<T: AsRef<Path>>(path: T, pattern: &str) -> Result<impl Iterator<Item = String>> {
    let path = path.as_ref();

    if !path.is_dir() {
        return Ok(Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _>>);
    }

    glob(pattern)
        .and_then(|paths| {
            let iter = paths.filter_map(|e| match e {
                Ok(it) => Some(it.to_string_lossy().into_owned()),
                Err(_) => None,
            });

            Ok(Box::new(iter) as Box<dyn Iterator<Item = _>>)
        })
        .map_err(Into::into)
}

pub fn list_match_filtered<T: AsRef<Path>, F: Fn(&PathBuf) -> bool + 'static>(
    path: T,
    pattern: &str,
    filter: F,
) -> Result<impl Iterator<Item = String>> {
    let path = path.as_ref();

    if !path.is_dir() {
        return Ok(Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _>>);
    }

    let filter = Rc::new(filter);
    let iter = match glob(pattern) {
        Ok(it) => it.filter_map({
            let filter = Rc::clone(&filter);
            move |e| match e {
                Ok(it) => {
                    if filter(&it) {
                        Some(it.to_string_lossy().into_owned())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        }),
        Err(_) => return Ok(Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _>>),
    };

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

pub fn remove<T: AsRef<Path>>(path: T) -> Result<()> {
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

pub fn remove_match<T: AsRef<Path>>(path: T, pattern: &str) -> Result<()> {
    let path = path.as_ref();

    if !path.is_dir() {
        return Ok(());
    }

    glob(pattern)
        .and_then(|paths| {
            for entry in paths.filter_map(|e| e.ok()) {
                if entry.is_dir() {
                    fs::remove_dir_all(entry).unwrap();
                } else {
                    fs::remove_file(entry).unwrap();
                }
            }

            Ok(())
        })
        .map_err(Into::into)
}

pub fn delete<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();

    if !path.is_dir() {
        return Ok(());
    }

    fs::remove_dir_all(path).map_err(Into::into)
}
