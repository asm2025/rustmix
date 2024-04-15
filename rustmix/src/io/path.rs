use std::path::{Path, PathBuf};

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

pub fn normalize<T: AsRef<str>>(path: T) -> String {
    let path = PathBuf::from(path.as_ref());
    path.to_string_lossy().to_string()
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
        .map(|e| e.to_string_lossy().to_string())
        .collect()
}

pub fn parent<T: AsRef<str>>(path: T) -> String {
    let path = PathBuf::from(path.as_ref());
    path.parent()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn name<T: AsRef<str>>(path: T) -> String {
    let path = PathBuf::from(path.as_ref());
    path.file_name()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn base_name<T: AsRef<str>>(path: T) -> String {
    let path = PathBuf::from(path.as_ref());
    path.file_stem()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn extension<T: AsRef<str>>(path: T) -> String {
    let path = PathBuf::from(path.as_ref());
    path.extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn set_extension<T: AsRef<str>>(path: T, ext: Option<&str>) -> String {
    let mut path = PathBuf::from(path.as_ref());

    if let Some(ext) = ext {
        path.set_extension(ext);
    } else {
        path.set_extension("");
    }

    path.to_string_lossy().to_string()
}
