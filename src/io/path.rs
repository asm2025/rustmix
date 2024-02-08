use std::path::PathBuf;

pub fn exists(path: &PathBuf) -> bool {
    path.exists()
}

pub fn take(path: &PathBuf, n: usize) -> PathBuf {
    let mut buffer = PathBuf::new();

    for component in path.components().take(n + 1) {
        buffer.push(component);
    }

    buffer
}

pub fn remove(path: &PathBuf, n: usize) -> PathBuf {
    let mut c: usize = 0;
    let mut buffer = PathBuf::from(path);

    while c < n && buffer.pop() {
        c += 1;
    }

    buffer
}

pub fn create(value: &str) -> PathBuf {
    let mut buffer = PathBuf::new();

    if !value.is_empty() {
        buffer.push(value);
    }

    buffer
}
