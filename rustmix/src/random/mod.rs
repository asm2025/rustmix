pub mod address;
pub mod automotive;
pub mod barcode;
pub mod business;
pub mod color;
pub mod datetime;
pub mod filesystem;
pub mod internet;
pub mod lorem;
pub mod person;

use fake::{faker::boolean::raw as f_boolean, locales, uuid, Fake};
use rand::{thread_rng, Rng};
use std::ops::Range;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UuidVersion {
    V1,
    V3,
    #[default]
    V4,
    V5,
}

pub fn alphanum() -> char {
    let mut rng = rand::thread_rng();
    let num = rng.gen_range(0..62);
    match num {
        0..=9 => char::from_u32(num as u32 + 48).unwrap(), // ASCII values for 0-9
        10..=35 => char::from_u32(num as u32 + 55).unwrap(), // ASCII values for A-Z
        _ => char::from_u32(num as u32 + 61).unwrap(),     // ASCII values for a-z
    }
}

pub fn alphanum_str(len: usize) -> String {
    if len == 0 {
        return String::new();
    }

    let mut s = String::with_capacity(len);

    for _ in 0..len {
        s.push(alphanum());
    }

    s
}

pub fn char() -> char {
    let mut rng = rand::thread_rng();
    let num = rng.gen_range(0..94);
    match num {
        0..=9 => char::from_u32(num as u32 + 48).unwrap(), // ASCII values for 0-9
        10..=35 => char::from_u32(num as u32 + 55).unwrap(), // ASCII values for A-Z
        36..=61 => char::from_u32(num as u32 + 61).unwrap(), // ASCII values for a-z
        _ => char::from_u32(num as u32 + 33).unwrap(),     // ASCII values for special characters
    }
}

pub fn string(len: usize) -> String {
    if len == 0 {
        return String::new();
    }

    let mut s = String::with_capacity(len);

    for _ in 0..len {
        s.push(char());
    }

    s
}

pub fn boolean() -> bool {
    f_boolean::Boolean(locales::EN, u8::MAX / 2).fake()
}

pub fn float() -> f64 {
    let mut rng = thread_rng();
    rng.gen_range(0.0..1.0)
}

pub fn numeric<T: rand::distributions::uniform::SampleUniform + PartialOrd>(r: Range<T>) -> T {
    let mut rng = thread_rng();
    rng.gen_range(r)
}

pub fn uuid() -> String {
    uuid::UUIDv4.fake()
}

pub fn uuid_v(version: UuidVersion) -> String {
    match version {
        UuidVersion::V1 => uuid::UUIDv1.fake(),
        UuidVersion::V3 => uuid::UUIDv3.fake(),
        UuidVersion::V5 => uuid::UUIDv5.fake(),
        _ => uuid::UUIDv4.fake(),
    }
}
