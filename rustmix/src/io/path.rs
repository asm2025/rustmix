pub use dirs::*;
pub use fs_extra::file::CopyOptions;
use fs_extra::{
    dir::{self as dirExtra, CopyOptions as DirCopyOptions},
    file as fileExtra,
};
use glob::glob_with;
pub use std::path::{Path, PathBuf};
use std::{fs, result::Result as StdResult};

use crate::{error::*, string::*, Result};

pub trait PathEx {
    fn as_str(&self) -> &str;
    fn exists(&self) -> bool;
    fn is_empty(&self) -> bool;
    fn take(&self, n: usize) -> PathBuf;
    fn remove(&self, n: usize) -> PathBuf;
}

impl<T: AsRef<Path>> PathEx for T {
    fn as_str(&self) -> &str {
        self.as_ref().to_str().unwrap_or_default()
    }

    fn exists(&self) -> bool {
        self.as_ref().exists()
    }

    fn is_empty(&self) -> bool {
        self.as_ref().as_os_str().is_empty()
    }

    fn take(&self, n: usize) -> PathBuf {
        let mut buffer = PathBuf::new();

        for component in self.as_ref().components().take(n) {
            buffer.push(component);
        }

        buffer
    }

    fn remove(&self, n: usize) -> PathBuf {
        let mut c: usize = 0;
        let mut path = PathBuf::from(self.as_ref());

        while c < n && path.pop() {
            c += 1;
        }

        path
    }
}

pub trait IntoPath<T> {
    fn into_path(&self) -> PathBuf;
}

impl<T: AsRef<str>> IntoPath<T> for T {
    fn into_path(&self) -> PathBuf {
        PathBuf::from(self.as_ref())
    }
}

impl<T: AsRef<str>> IntoPath<T> for (T, T) {
    fn into_path(&self) -> PathBuf {
        let mut path = PathBuf::new();
        append_if_not_empty(&mut path, self.0.as_ref());
        append_if_not_empty(&mut path, self.1.as_ref());
        path
    }
}

impl<T: AsRef<str>> IntoPath<T> for (T, T, T) {
    fn into_path(&self) -> PathBuf {
        let mut path = PathBuf::new();
        append_if_not_empty(&mut path, self.0.as_ref());
        append_if_not_empty(&mut path, self.1.as_ref());
        append_if_not_empty(&mut path, self.2.as_ref());
        path
    }
}

impl<T: AsRef<str>> IntoPath<T> for (T, T, T, T) {
    fn into_path(&self) -> PathBuf {
        let mut path = PathBuf::new();
        append_if_not_empty(&mut path, self.0.as_ref());
        append_if_not_empty(&mut path, self.1.as_ref());
        append_if_not_empty(&mut path, self.2.as_ref());
        append_if_not_empty(&mut path, self.3.as_ref());
        path
    }
}

impl<T: AsRef<str>> IntoPath<T> for (T, T, T, T, T) {
    fn into_path(&self) -> PathBuf {
        let mut path = PathBuf::new();
        append_if_not_empty(&mut path, self.0.as_ref());
        append_if_not_empty(&mut path, self.1.as_ref());
        append_if_not_empty(&mut path, self.2.as_ref());
        append_if_not_empty(&mut path, self.3.as_ref());
        append_if_not_empty(&mut path, self.4.as_ref());
        path
    }
}

impl<T: AsRef<str>, const N: usize> IntoPath<T> for [T; N] {
    fn into_path(&self) -> PathBuf {
        let mut path = PathBuf::new();

        if self.is_empty() {
            return path;
        }

        for component in self.iter() {
            append_if_not_empty(&mut path, component.as_ref());
        }

        path
    }
}

impl<T: AsRef<str>> IntoPath<T> for Vec<T> {
    fn into_path(&self) -> PathBuf {
        let mut path = PathBuf::new();

        if self.is_empty() {
            return path;
        }

        for component in self.iter() {
            append_if_not_empty(&mut path, component.as_ref());
        }

        path
    }
}

pub trait AsPath<T> {
    fn as_path(&self) -> String;
    fn as_full_path(&self) -> String;
}

impl<T: AsRef<str>> AsPath<T> for (T, T) {
    fn as_path(&self) -> String {
        let path = self.into_path();
        path.as_str().to_string()
    }

    fn as_full_path(&self) -> String {
        let path = self.into_path();
        path.canonicalize().unwrap().as_str().to_string()
    }
}

impl<T: AsRef<str>> AsPath<T> for (T, T, T) {
    fn as_path(&self) -> String {
        let path = self.into_path();
        path.as_str().to_string()
    }

    fn as_full_path(&self) -> String {
        let path = self.into_path();
        path.canonicalize().unwrap().as_str().to_string()
    }
}

impl<T: AsRef<str>> AsPath<T> for (T, T, T, T) {
    fn as_path(&self) -> String {
        let path = self.into_path();
        path.as_str().to_string()
    }

    fn as_full_path(&self) -> String {
        let path = self.into_path();
        path.canonicalize().unwrap().as_str().to_string()
    }
}

impl<T: AsRef<str>> AsPath<T> for (T, T, T, T, T) {
    fn as_path(&self) -> String {
        let path = self.into_path();
        path.as_str().to_string()
    }

    fn as_full_path(&self) -> String {
        let path = self.into_path();
        path.canonicalize().unwrap().as_str().to_string()
    }
}

impl<T: AsRef<str>, const N: usize> AsPath<T> for [T; N] {
    fn as_path(&self) -> String {
        let path = self.into_path();
        path.as_str().to_string()
    }

    fn as_full_path(&self) -> String {
        let path = self.into_path();
        path.canonicalize().unwrap().as_str().to_string()
    }
}

impl<T: AsRef<str>> AsPath<T> for Vec<T> {
    fn as_path(&self) -> String {
        let path = self.into_path();
        path.as_str().to_string()
    }

    fn as_full_path(&self) -> String {
        let path = self.into_path();
        path.canonicalize().unwrap().as_str().to_string()
    }
}

fn append_if_not_empty(path: &mut PathBuf, component: &str) {
    if !component.is_empty() {
        path.push(component);
    }
}

fn file_options_to_dir_options(options: &CopyOptions) -> DirCopyOptions {
    DirCopyOptions {
        overwrite: options.overwrite,
        skip_exist: options.skip_exist,
        buffer_size: options.buffer_size,
        ..Default::default()
    }
}

fn glob_defaults() -> glob::MatchOptions {
    glob::MatchOptions {
        case_sensitive: false,
        ..Default::default()
    }
}

pub fn normalize<T: AsRef<str>>(path: T) -> String {
    let path = PathBuf::from(path.as_ref());
    path.to_string_lossy().into_owned()
}

pub fn get_full_path<T: AsRef<str>>(path: T) -> String {
    let path = PathBuf::from(path.as_ref());
    path.canonicalize()
        .unwrap_or_else(|_| path)
        .to_string_lossy()
        .to_string()
}

pub fn is_absolute<T: AsRef<str>>(path: T) -> bool {
    Path::new(path.as_ref()).is_absolute()
}

pub fn is_relative<T: AsRef<str>>(path: T) -> bool {
    Path::new(path.as_ref()).is_relative()
}

pub fn has_separator<T: AsRef<str>>(path: T) -> bool {
    path.as_ref().contains(std::path::is_separator)
}

pub fn split<T: AsRef<str>>(path: T) -> Vec<String> {
    let path = PathBuf::from(path.as_ref());
    path.iter()
        .filter(|e| !e.is_empty())
        .map(|e| e.to_string_lossy().into_owned())
        .collect()
}

pub fn parent<T: AsRef<str>>(path: T) -> String {
    let path = PathBuf::from(path.as_ref());
    path.parent()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub fn name<T: AsRef<str>>(path: T) -> String {
    let path = PathBuf::from(path.as_ref());
    path.file_name()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub fn base_name<T: AsRef<str>>(path: T) -> String {
    let path = PathBuf::from(path.as_ref());
    path.file_stem()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub fn extension<T: AsRef<str>>(path: T) -> String {
    let path = PathBuf::from(path.as_ref());
    path.extension()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub fn set_extension<T: AsRef<str>>(path: T, ext: Option<&str>) -> String {
    let mut path = PathBuf::from(path.as_ref());

    if let Some(ext) = ext {
        path.set_extension(ext);
    } else {
        path.set_extension("");
    }

    path.to_string_lossy().into_owned()
}

pub fn lst<T: AsRef<Path>>(path: T) -> Result<impl Iterator<Item = PathBuf>> {
    let path = path.as_ref();

    if !path.is_dir() {
        return Ok(Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _>>);
    }

    let read_dir = fs::read_dir(path)?;
    let iter = read_dir.filter_map(|e| match e {
        Ok(entry) => Some(entry.path()),
        Err(_) => None,
    });
    Ok(Box::new(iter) as Box<dyn Iterator<Item = _>>)
}

pub fn lst_filtered<T: AsRef<Path>, F: Fn(&PathBuf) -> bool + 'static>(
    path: T,
    filter: F,
) -> Result<impl Iterator<Item = PathBuf>> {
    let path = path.as_ref();

    if !path.is_dir() {
        if !path.exists() {
            let path = path.to_string_lossy().into_owned();
            return Err(NotFoundError(path).into());
        }

        return Ok(Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _>>);
    }

    let read_dir = fs::read_dir(path)?;
    let iter = read_dir.filter_map(move |e| match e {
        Ok(entry) => {
            let path = entry.path();

            if filter(&path) {
                Some(path)
            } else {
                None
            }
        }
        Err(_) => None,
    });
    Ok(Box::new(iter) as Box<dyn Iterator<Item = _>>)
}

pub fn lst_match<T: AsRef<str>>(pattern: T) -> Result<impl Iterator<Item = PathBuf>> {
    let pattern = pattern.as_ref();

    if pattern.is_empty() {
        return Ok(Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _>>);
    }

    let paths = glob_with(&pattern, glob_defaults())?;
    let iter = paths.filter_map(StdResult::ok);

    Ok(Box::new(iter) as Box<dyn Iterator<Item = _>>)
}

pub fn lst_match_filtered<T: AsRef<str>, F: Fn(&PathBuf) -> bool + 'static>(
    pattern: T,
    filter: F,
) -> Result<impl Iterator<Item = PathBuf>> {
    let pattern = pattern.as_ref();

    if pattern.is_empty() {
        return Ok(Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _>>);
    }

    let paths = glob_with(&pattern, glob_defaults())?;
    let iter = paths.filter_map(StdResult::ok).filter(move |e| filter(e));

    Ok(Box::new(iter) as Box<dyn Iterator<Item = _>>)
}

pub fn cpy<F: AsRef<str>, T: AsRef<Path>>(from: F, to: T) -> Result<()> {
    cpy_with(from, to, &CopyOptions::new())
}

pub fn cpy_with<F: AsRef<str>, T: AsRef<Path>>(from: F, to: T, option: &CopyOptions) -> Result<()> {
    let from = from.as_ref();

    if from.is_empty() {
        return Err(InvalidOperationError("Empty source path".to_string()).into());
    }

    let to = to.as_ref();

    if from.find_first(|e| e == '*' || e == '?').is_some() {
        return use_wild_card(from, to, option).map_err(Into::into);
    }

    let from = Path::new(from);
    return use_path(from, to, option).map_err(Into::into);

    fn use_path(from: &Path, to: &Path, option: &CopyOptions) -> Result<()> {
        fs::create_dir_all(&to)?;

        if from.is_dir() {
            let dir_options = file_options_to_dir_options(option);
            dirExtra::copy(from, to, &dir_options)?;
        } else {
            fileExtra::copy(from, to, option)?;
        }

        Ok(())
    }

    fn use_wild_card(from: &str, to: &Path, option: &CopyOptions) -> Result<()> {
        fs::create_dir_all(&to)?;
        let paths = glob_with(from, glob_defaults())?;
        let dir_options = file_options_to_dir_options(option);

        for entry in paths.filter_map(|e| e.ok()) {
            let desination = to.join(entry.file_name().unwrap());

            if entry.is_dir() {
                dirExtra::copy(entry, desination, &dir_options)?;
            } else {
                fileExtra::copy(entry, desination, option)?;
            }
        }

        Ok(())
    }
}

pub fn mov<F: AsRef<str>, T: AsRef<Path>>(from: F, to: T) -> Result<()> {
    mov_with(from, to, &CopyOptions::new())
}

pub fn mov_with<F: AsRef<str>, T: AsRef<Path>>(from: F, to: T, option: &CopyOptions) -> Result<()> {
    let from = from.as_ref();

    if from.is_empty() {
        return Err(InvalidOperationError("Empty source path".to_string()).into());
    }

    let to = to.as_ref();

    if from.find_first(|e| e == '*' || e == '?').is_some() {
        return use_wild_card(from, to, option).map_err(Into::into);
    }

    let from = Path::new(from);
    return use_path(from, to, option).map_err(Into::into);

    fn use_path(from: &Path, to: &Path, option: &CopyOptions) -> Result<()> {
        fs::create_dir_all(&to)?;

        if from.is_dir() {
            let dir_options = file_options_to_dir_options(option);
            dirExtra::move_dir(from, to, &dir_options)?;
        } else {
            fileExtra::move_file(from, to, option)?;
        }

        Ok(())
    }

    fn use_wild_card(from: &str, to: &Path, option: &CopyOptions) -> Result<()> {
        fs::create_dir_all(&to)?;
        let paths = glob_with(from, glob_defaults())?;
        let dir_options = file_options_to_dir_options(option);

        for entry in paths.filter_map(|e| e.ok()) {
            let desination = to.join(entry.file_name().unwrap());

            if entry.is_dir() {
                dirExtra::move_dir(entry, desination, &dir_options)?;
            } else {
                fileExtra::move_file(entry, desination, option)?;
            }
        }

        Ok(())
    }
}

pub fn ren<F: AsRef<Path>, T: AsRef<str>>(from: F, to: T) -> Result<()> {
    let to = to.as_ref();

    if to.contains(std::path::is_separator) {
        let to = Path::new(to);
        return fs::rename(from, to).map_err(Into::into);
    }

    let from = from.as_ref();
    let parent = from.parent().unwrap();
    let to = parent.join(to);
    fs::rename(from, to).map_err(Into::into)
}

pub fn del<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();

    if path.is_dir() {
        fs::remove_dir_all(path).map_err(Into::into)
    } else {
        fs::remove_file(path).map_err(Into::into)
    }
}

pub fn del_match<T: AsRef<Path>>(path: T, pattern: &str) -> Result<()> {
    let path = path.as_ref();

    if !path.is_dir() {
        if path.is_file() {
            return Err(InvalidDirectoryError(path.to_string_lossy().into_owned()).into());
        }
        return Ok(());
    }

    if pattern.is_empty() {
        return Err(InvalidOperationError("Empty source path".to_string()).into());
    }

    let pattern = format!("{}/{}", path.to_string_lossy(), pattern);
    let paths = glob_with(&pattern, glob_defaults())?;

    for entry in paths.filter_map(|e| e.ok()) {
        if entry.is_dir() {
            fs::remove_dir_all(entry)?;
        } else {
            fs::remove_file(entry)?;
        }
    }

    Ok(())
}
