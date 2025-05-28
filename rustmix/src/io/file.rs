use csv::{ReaderBuilder, WriterBuilder};
use serde::{de, Serialize};
use serde_json;
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
};

use super::directory;
use crate::Result;

const LINES_BUFFER_DEFAULT: usize = 1000;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileOpenOptions {
    #[default]
    Default,
    New,
    Truncate,
    Append,
}

pub fn exists<T: AsRef<Path>>(path: T) -> bool {
    path.as_ref().is_file()
}

pub fn open<T: AsRef<Path>>(path: T) -> Result<std::fs::File> {
    let mut opt = OpenOptions::new();
    opt.read(true);
    from_options(path, &opt)
}

pub fn create<T: AsRef<Path>>(path: T) -> Result<std::fs::File> {
    create_with(path, FileOpenOptions::Default)
}

pub fn create_with<T: AsRef<Path>>(path: T, options: FileOpenOptions) -> Result<std::fs::File> {
    let path = path.as_ref();
    let dir = path.parent().unwrap();
    directory::ensure(dir)?;

    let mut opt = OpenOptions::new();
    opt.read(true);
    match options {
        FileOpenOptions::New => opt.create_new(true),
        FileOpenOptions::Truncate => opt.create(true).truncate(true),
        _ => opt.create(true).append(true),
    };
    opt.write(true);
    from_options(path, &opt)
}

pub fn from_options<T: AsRef<Path>>(path: T, options: &OpenOptions) -> Result<std::fs::File> {
    let path = path.as_ref();
    options.open(path).map_err(Into::into)
}

pub fn delete<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path).map_err(Into::into)
}

pub trait FileEx {
    fn read(&self) -> Result<impl Iterator<Item = String>>;
    fn read_filtered<F: Fn(&str) -> bool + 'static>(
        &self,
        filter: F,
    ) -> Result<impl Iterator<Item = String>>;
    fn read_batch<R: Fn(u32, Vec<String>) -> bool + 'static>(
        &self,
        batch: usize,
        callback: R,
    ) -> Result<u32>;
    fn read_batch_filtered<
        F: Fn(&str) -> bool + 'static,
        R: Fn(u32, Vec<String>) -> bool + 'static,
    >(
        &self,
        batch: usize,
        filter: F,
        callback: R,
    ) -> Result<u32>;
    fn write<T: AsRef<str>>(&mut self, data: &T) -> Result<()>;
    fn write_lines<T: AsRef<str>>(&mut self, data: impl Iterator<Item = T>) -> Result<()>;
    fn read_json<T: de::DeserializeOwned>(&self) -> Result<T>;
    fn write_json<T: Serialize>(&mut self, data: &T, pretty: Option<bool>) -> Result<()>;
    fn create_delimited_reader(
        &mut self,
        delimiter: Option<u8>,
        has_headers: Option<bool>,
    ) -> csv::Reader<&mut std::fs::File>;
    fn create_delimited_writer(
        &mut self,
        delimiter: Option<u8>,
        has_headers: Option<bool>,
    ) -> csv::Writer<&mut std::fs::File>;
}

impl FileEx for std::fs::File {
    fn read(&self) -> Result<impl Iterator<Item = String>> {
        let reader = BufReader::new(self);
        Ok(reader.lines().filter_map(|line| line.ok()))
    }

    fn read_filtered<F: Fn(&str) -> bool + 'static>(
        &self,
        filter: F,
    ) -> Result<impl Iterator<Item = String>> {
        let reader = BufReader::new(self);
        Ok(reader.lines().filter_map(move |e| {
            if let Ok(line) = e {
                if line.is_empty() || !filter(&line) {
                    return None;
                }

                Some(line)
            } else {
                None
            }
        }))
    }

    fn read_batch<R: Fn(u32, Vec<String>) -> bool + 'static>(
        &self,
        batch: usize,
        callback: R,
    ) -> Result<u32> {
        let batch = if batch == 0 {
            LINES_BUFFER_DEFAULT
        } else {
            batch
        };
        let mut reader = BufReader::new(self);
        let mut batch_number = 0u32;
        let mut line: String = String::new();
        let mut lines: Vec<String> = Vec::with_capacity(batch);

        while let Ok(n) = reader.read_line(&mut line) {
            if n == 0 {
                break;
            }

            if line.is_empty() {
                line.clear();
                continue;
            }

            lines.push(line.clone());
            line.clear();

            if lines.len() < batch {
                continue;
            }

            batch_number += 1;
            let contin = callback(batch_number, lines.clone());
            lines.clear();

            if !contin {
                break;
            }
        }

        if lines.is_empty() {
            return Ok(batch_number);
        }

        batch_number += 1;
        callback(batch_number, lines.clone());
        Ok(batch_number)
    }

    fn read_batch_filtered<
        F: Fn(&str) -> bool + 'static,
        R: Fn(u32, Vec<String>) -> bool + 'static,
    >(
        &self,
        batch: usize,
        filter: F,
        callback: R,
    ) -> Result<u32> {
        let batch = if batch == 0 {
            LINES_BUFFER_DEFAULT
        } else {
            batch
        };
        let mut reader = BufReader::new(self);
        let mut batch_number = 0u32;
        let mut line: String = String::new();
        let mut lines = Vec::with_capacity(batch);

        while let Ok(n) = reader.read_line(&mut line) {
            if n == 0 {
                break;
            }

            if line.is_empty() || !filter(&line) {
                line.clear();
                continue;
            }

            lines.push(line.clone());
            line.clear();

            if lines.len() < batch {
                continue;
            }

            batch_number += 1;
            let contin = callback(batch_number, lines.clone());
            lines.clear();

            if !contin {
                break;
            }
        }

        if lines.is_empty() {
            return Ok(batch_number);
        }

        batch_number += 1;
        callback(batch_number, lines.clone());
        Ok(batch_number)
    }

    fn write<T: AsRef<str>>(&mut self, data: &T) -> Result<()> {
        writeln!(self, "{}", data.as_ref()).map_err(Into::into)
    }

    fn write_lines<T: AsRef<str>>(&mut self, data: impl Iterator<Item = T>) -> Result<()> {
        for line in data.into_iter() {
            writeln!(self, "{}", line.as_ref())?;
        }

        Ok(())
    }

    fn read_json<T: de::DeserializeOwned>(&self) -> Result<T> {
        let reader = BufReader::new(self);
        let data: T = serde_json::from_reader(reader)?;
        Ok(data)
    }

    fn write_json<T: Serialize>(&mut self, data: &T, pretty: Option<bool>) -> Result<()> {
        let serialize = match pretty {
            Some(true) => serde_json::to_string_pretty,
            _ => serde_json::to_string,
        };
        let serialized = serialize(data)?;
        self.write_all(serialized.as_bytes())?;
        Ok(())
    }

    fn create_delimited_reader(
        &mut self,
        delimiter: Option<u8>,
        has_headers: Option<bool>,
    ) -> csv::Reader<&mut std::fs::File> {
        let delimiter = delimiter.unwrap_or(b',');
        let has_headers = has_headers.unwrap_or(false);
        ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(has_headers)
            .from_reader(self)
    }

    fn create_delimited_writer(
        &mut self,
        delimiter: Option<u8>,
        has_headers: Option<bool>,
    ) -> csv::Writer<&mut std::fs::File> {
        let delimiter = delimiter.unwrap_or(b',');
        let has_headers = has_headers.unwrap_or(false);
        WriterBuilder::new()
            .delimiter(delimiter)
            .has_headers(has_headers)
            .from_writer(self)
    }
}
