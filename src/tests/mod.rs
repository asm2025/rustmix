#![allow(unused_imports)]

use serde::{Deserialize, Serialize};
use std::io::Write;

mod app;
pub(crate) use self::app::*;
mod io;
pub(crate) use self::io::*;
#[cfg(feature = "log4rs")]
mod log4rs;
#[cfg(feature = "log4rs")]
pub(crate) use self::log4rs::*;
#[cfg(feature = "mail")]
mod mail;
#[cfg(feature = "mail")]
pub(crate) use self::mail::*;
#[cfg(feature = "python")]
mod python;
#[cfg(feature = "python")]
pub(crate) use self::python::*;
#[cfg(feature = "slog")]
mod slog;
#[cfg(feature = "slog")]
pub(crate) use self::slog::*;
#[cfg(feature = "threading")]
mod threading;
#[cfg(feature = "threading")]
pub(crate) use self::threading::*;
mod web;
pub(crate) use self::web::*;
#[cfg(feature = "audio")]
mod audio;
#[cfg(feature = "audio")]
pub(crate) use self::audio::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Employee {
    id: u32,
    #[serde(rename = "employee_name")]
    name: String,
    #[serde(rename = "employee_age")]
    age: u8,
    email: Option<String>,
    phone: Option<String>,
    #[serde(rename = "profile_image")]
    image: Option<String>,
    #[serde(rename = "employee_salary")]
    salary: Option<f64>,
}

pub fn get_employees(count: usize) -> Vec<Employee> {
    let n = if count < 1 { 1 } else { count };
    let mut employees: Vec<Employee> = Vec::with_capacity(n);

    for i in 1..=n {
        let name = format!("Employee {}", i);
        employees.push(Employee {
            id: i as u32,
            name,
            age: (i % 100) as u8,
            email: Some(format!("{}@example.com", i)),
            phone: Some(format!("+1-555-555-{:04}", i)),
            image: Some(format!("https://i.pravatar.cc/150?img={}", i)),
            salary: Some((i * 1000) as f64),
        });
    }

    employees
}

pub fn print_batch<T: std::fmt::Display>(batch: u32, items: Vec<T>) -> bool {
    println!("\nReading batch {}", batch);

    for item in items {
        print!("{}", item);
    }

    true
}

pub fn stdin_input(prompt: &str) -> String {
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
