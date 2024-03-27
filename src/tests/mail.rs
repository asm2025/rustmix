use anyhow::Result;
use std::io::{self, Write};

use rustmix::mail::{emailfake::EmailFake, secmail::SecMail, tempmail::TempMail};

const PREFIX: &str = "My search string is ";
const PREFIX_LEN: usize = 14;

pub async fn test_secmail() -> Result<()> {
    println!("\nTesting SecMail functions...");
    println!("The supported domains for this class are:");
    println!("{}", SecMail::get_domains().join(", "));
    print!("Enter the email address [ENTER to generate]: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let email = if input.is_empty() {
        SecMail::random()
    } else {
        SecMail::parse(input)
    };

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
    print!("Enter the email [ENTER to generate]: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let email = if input.is_empty() {
        EmailFake::random().await?
    } else {
        EmailFake::parse(input)
    };

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
    println!("\nTesting Tempmail functions...");
    print!("Enter the email [ENTER to generate]: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let email = if input.is_empty() {
        TempMail::random().await?
    } else {
        TempMail::parse(input)
    };

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
