use std::path::{Path, PathBuf};

pub trait PathExt {
    fn as_str(&self) -> &str;
    fn exists(&self) -> bool;
    fn is_empty(&self) -> bool;
    fn take(&self, n: usize) -> PathBuf;
    fn remove(&self, n: usize) -> PathBuf;
}

pub trait IntoPath<T> {
    fn into_path(&self) -> PathBuf;
}

pub trait AsPath<T> {
    fn as_path(&self) -> String;
}

impl<T: AsRef<Path>> PathExt for T {
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

fn append_if_not_empty(path: &mut PathBuf, component: &str) {
    if !component.is_empty() {
        path.push(component);
    }
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

impl<T: AsRef<str>> IntoPath<T> for [T] {
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

impl<T: AsRef<str>> AsPath<T> for (T, T) {
    fn as_path(&self) -> String {
        let path = self.into_path();
        path.as_str().to_string()
    }
}

impl<T: AsRef<str>> AsPath<T> for (T, T, T) {
    fn as_path(&self) -> String {
        let path = self.into_path();
        path.as_str().to_string()
    }
}

impl<T: AsRef<str>> AsPath<T> for (T, T, T, T) {
    fn as_path(&self) -> String {
        let path = self.into_path();
        path.as_str().to_string()
    }
}

impl<T: AsRef<str>> AsPath<T> for (T, T, T, T, T) {
    fn as_path(&self) -> String {
        let path = self.into_path();
        path.as_str().to_string()
    }
}

impl<T: AsRef<str>> AsPath<T> for [T] {
    fn as_path(&self) -> String {
        let path = self.into_path();
        path.as_str().to_string()
    }
}

impl<T: AsRef<str>> AsPath<T> for Vec<T> {
    fn as_path(&self) -> String {
        let path = self.into_path();
        path.as_str().to_string()
    }
}

pub fn split_path<T: AsRef<str>>(path: T) -> Vec<String> {
    let path = PathBuf::from(path.as_ref());
    path.iter()
        .filter(|e| !e.is_empty())
        .map(|e| e.to_string_lossy().to_string())
        .collect()
}
