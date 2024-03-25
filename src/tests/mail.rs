use anyhow::Result;
use std::io::{self, Write};

use rustmix::mail::{emailfake::EmailFake, secmail::SecMail, tempmail::TempMail};

const PREFIX: &str = "My OTP is ";
const PREFIX_LEN: usize = 14;

pub async fn test_secmail() -> Result<()> {
    println!("\nTesting SecMail functions...");

    let mut email = SecMail::random();
    print!("Enter the email [default: {}]: ", email.address());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if !input.is_empty() {
        email = SecMail::parse(input);
    }

    print!("Enter the sender email [default: None]: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    let from = if input.is_empty() { None } else { Some(input) };
    println!(
        "Email: {}, from sender: {}",
        email.address(),
        from.unwrap_or("None")
    );

    let str = email.find_string(from, None, PREFIX, PREFIX_LEN).await?;
    println!("My string: {}", str);

    Ok(())
}

pub async fn test_emailfake() -> Result<()> {
    println!("\nTesting EmailFake functions...");

    let mut email = EmailFake::random().await?;
    print!("Enter the email [default: {}]: ", email.address());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if !input.is_empty() {
        email = EmailFake::parse(input);
    }

    print!("Enter the sender email [default: None]: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    let from = if input.is_empty() { None } else { Some(input) };
    println!(
        "Email: {}, from sender: {}",
        email.address(),
        from.unwrap_or("None")
    );

    let str = email.find_string(from, None, PREFIX, PREFIX_LEN).await?;
    println!("My string: {}", str);

    Ok(())
}

pub async fn test_tempmail() -> Result<()> {
    println!("\nTesting TempMail functions...");

    let mut email = TempMail::random().await?;
    print!("Enter the email [default: {}]: ", email.address());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if !input.is_empty() {
        email = TempMail::parse(input);
    }

    print!("Enter the sender email [default: None]: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    let from = if input.is_empty() { None } else { Some(input) };
    println!(
        "Email: {}, from sender: {}",
        email.address(),
        from.unwrap_or("None")
    );

    let str = email.find_string(from, None, PREFIX, PREFIX_LEN).await?;
    println!("My string: {}", str);

    Ok(())
}
