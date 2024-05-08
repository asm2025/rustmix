use std::{
    io::{stdin, stdout, Write},
    str::FromStr,
};
use termion::{event::Key, input::TermRead};

use crate::{
    error::{InvalidInputError, NoInputError, NotConfirmError},
    Result,
};

pub fn get(prompt: &str) -> Result<String> {
    if !prompt.is_empty() {
        print!("{}", prompt);
        stdout().flush()?;
    }

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    input.pop(); // Remove the trailing newlines

    if input.is_empty() {
        Err(NoInputError.into())
    } else {
        Ok(input)
    }
}

pub fn get_char(prompt: &str) -> Result<char> {
    if !prompt.is_empty() {
        print!("{}", prompt);
        stdout().flush()?;
    }

    let mut input = stdin().keys();

    if let Some(Ok(Key::Char(c))) = input.next() {
        return Ok(c);
    }

    Err(NoInputError.into())
}

pub fn get_numeric<T: FromStr>(prompt: &str) -> Result<T>
where
    <T as FromStr>::Err: std::fmt::Display,
{
    let input = get(prompt)?;
    input.parse::<T>().map_err(|_| InvalidInputError.into())
}

pub fn get_password(prompt: &str) -> Result<String> {
    if !prompt.is_empty() {
        print!("{}", prompt);
        stdout().flush()?;
    }

    let input = rpassword::read_password()?;

    if input.is_empty() {
        Err(NoInputError.into())
    } else {
        Ok(input)
    }
}

pub fn confirm(prompt: &str) -> Result<bool> {
    let input = get_char(prompt)?;

    match input {
        'y' | 'Y' => Ok(true),
        _ => Err(NotConfirmError.into()),
    }
}
