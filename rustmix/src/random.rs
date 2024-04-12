pub use fake::{faker::*, uuid::*, Fake};
use rand::{thread_rng, Rng};

use crate::string::SPECIAL_CHARS;

pub fn string(len: usize) -> String {
    let mut s = String::with_capacity(len);

    for _ in 0..len {
        s.push(alphanum());
    }

    s
}

pub fn alphanum() -> char {
    let c = rand::thread_rng().gen_range(0..62);

    if c < 10 {
        (char::from_digit(c, 10).unwrap() as u8 + 48) as char
    } else if c < 36 {
        (char::from_digit(c - 10, 10).unwrap() as u8 + 65) as char
    } else {
        (char::from_digit(c - 36, 10).unwrap() as u8 + 97) as char
    }
}

pub fn char() -> char {
    let mut rnd = rand::thread_rng();
    let c = rnd.gen_range(0..62);

    if c < 10 {
        (char::from_digit(c, 10).unwrap() as u8 + 48) as char
    } else if c < 36 {
        (char::from_digit(c - 10, 10).unwrap() as u8 + 65) as char
    } else if c < 62 {
        (char::from_digit(c - 36, 10).unwrap() as u8 + 97) as char
    } else {
        SPECIAL_CHARS[rnd.gen_range(0..10)]
    }
}

pub fn float() -> f64 {
    let mut rng = thread_rng();
    rng.gen_range(0.0..1.0)
}

pub fn numeric<T>(min: T, max: T) -> T
where
    T: rand::distributions::uniform::SampleUniform + PartialOrd,
{
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

pub fn boolean() -> bool {
    let mut rng = thread_rng();
    rng.gen_bool(0.5)
}
