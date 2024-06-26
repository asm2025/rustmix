use rustmix::{
    date::{parse_date_any, DATE_FORMAT, DATE_TIME_FORMAT},
    input,
    web::mail::{TempMail, TempMailProvider},
    Result,
};

const PREFIX: &str = "My search string is ";
const PREFIX_LEN: usize = 14;

pub async fn test_tempmail() -> Result<()> {
    println!("\nTesting TempMail functions...");
    let input = input::get("Enter the email [ENTER to generate]: ")?;

    let email = if input.is_empty() {
        TempMail::random().await?
    } else {
        TempMail::parse(TempMailProvider::Tempmail, &input)
    };

    let input = input::get("Enter the email's sender [default: None]: ")?;
    let from = if input.is_empty() {
        None
    } else {
        Some(input.as_str())
    };

    let input = input::get("Enter the email's subject [default: None]: ")?;
    let subject = if input.is_empty() {
        None
    } else {
        Some(input.as_str())
    };

    let input = input::get(&format!(
        "Enter the email's date in either format: '{}' or '{}' [default: None]: ",
        DATE_FORMAT, DATE_TIME_FORMAT
    ))?;
    let date = if input.is_empty() {
        None
    } else {
        Some(parse_date_any(&input)?)
    };
    println!(
        "Email: {}, from sender: {:?}, subject: {:?}, date: {:?}",
        email.address(),
        from,
        subject,
        date
    );

    let str = email
        .find_string(from, subject, date, PREFIX, PREFIX_LEN)
        .await?;
    println!("My string: {}", str);

    Ok(())
}
