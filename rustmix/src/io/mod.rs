pub mod directory;
pub mod file;
pub mod path;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use dialoguer::{theme::ColorfulTheme, Select};
use std::{
    io::{stdin, stdout, Write},
    str::FromStr,
};

use crate::{error::RmxError, Result};

pub fn clear_screen() -> Result<()> {
    let mut stdout = stdout();
    stdout
        .execute(Clear(ClearType::All))?
        .execute(cursor::MoveTo(0, 0))?;
    Ok(())
}

pub fn display_menu(items: &[&str], prompt: Option<&str>) -> Result<usize> {
    clear_screen()?;

    let prompt = match prompt {
        Some(s) if !s.is_empty() => s,
        _ => "Please select an option",
    };
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()
        .map_err(|e| RmxError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok(if selection == items.len() - 1 {
        0
    } else {
        selection + 1
    })
}

pub fn get(prompt: Option<&str>) -> Result<String> {
    print_prompt(prompt);

    let mut buffer = String::new();
    stdin().read_line(&mut buffer)?;

    if !buffer.is_empty() {
        // Remove the trailing newlines
        buffer.pop();
    }

    Ok(buffer)
}

pub fn get_str(prompt: Option<&str>) -> Result<String> {
    let input = get(prompt)?;

    if input.is_empty() {
        return Err(RmxError::NoInput);
    }

    Ok(input)
}

pub fn get_char(prompt: Option<&str>) -> Result<char> {
    print_prompt(prompt);
    // Enable raw mode to read single characters
    enable_raw_mode()?;

    let result = loop {
        if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Char(c) => break Ok(c),
                KeyCode::Esc | KeyCode::Enter => break Err(RmxError::NoInput),
                _ => continue,
            }
        }
    };

    // Disable raw mode before returning
    disable_raw_mode()?;
    result
}

pub fn get_numeric<T: FromStr>(prompt: Option<&str>) -> Result<T>
where
    <T as FromStr>::Err: std::fmt::Display,
    T::Err: std::error::Error + 'static,
{
    let input = get_str(prompt)?;
    let n = input.parse::<T>().map_err(|e| {
        RmxError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            e.to_string(),
        ))
    })?;
    Ok(n)
}

pub fn get_password(prompt: Option<&str>) -> Result<String> {
    print_prompt(prompt);

    let input = rpassword::read_password()?;
    Ok(input)
}

pub fn get_password_str(prompt: Option<&str>) -> Result<String> {
    let input = get_password(prompt)?;

    if input.is_empty() {
        return Err(RmxError::NoInput);
    }

    Ok(input)
}

pub fn confirm(prompt: Option<&str>) -> Result<bool> {
    let input = get_char(prompt)?;

    match input {
        'y' | 'Y' => Ok(true),
        _ => Err(RmxError::NoInput),
    }
}

pub fn pause() {
    println!("Press any key to continue...");
    get_char(None).unwrap();
}

fn print_prompt(prompt: Option<&str>) {
    if let Some(p) = prompt {
        if !p.is_empty() {
            print!("{} ", p);
            stdout().flush().expect("Failed to flush stdout");
        }
    }
}
