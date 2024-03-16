use serde::{Deserialize, Serialize};

pub mod io;
pub mod mail;
pub mod python;
pub mod threading;
pub mod web;

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
