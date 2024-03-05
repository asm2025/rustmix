use fake::{
    faker::{internet::raw::*, lorem::raw::*, name::raw::*, phone_number::raw::*},
    locales::EN,
    Dummy, Fake, Faker,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub mod io;
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

impl Dummy<Faker> for Employee {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        Employee {
            id: rng.gen_range(1..10000),
            name: Name(EN).fake(),
            age: rng.gen_range(18..65),
            email: Some(SafeEmail(EN).fake()),
            phone: Some(PhoneNumber(EN).fake()),
            image: Some(format!(
                "https://i.pravatar.cc/150?img={}",
                Word(EN).fake::<String>()
            )),
            salary: Some(rng.gen_range(1000.0..100000.0)),
        }
    }
}

pub fn get_employees(count: usize) -> Vec<Employee> {
    let n = if count < 1 { 1 } else { count };
    (0..n).map(|_| Faker.fake()).collect()
}

pub fn print_batch<T: std::fmt::Display>(batch: u32, items: Vec<T>) -> bool {
    println!("\nReading batch {}", batch);

    for item in items {
        print!("{}", item);
    }

    true
}
