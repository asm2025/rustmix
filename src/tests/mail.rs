use std::{
    error::Error,
    io::{self, Write},
};

use rustmix::mail::tempmail::Tempmail;

const EMAIL: &str = "someone@example.com";
const OTP: &str = "My OTP is ";
const OTP_LEN: usize = 6;

pub async fn test_tmp_mail() -> Result<(), Box<dyn Error>> {
    println!("\nTesting Temp Mail functions...");

    let email = Tempmail::new();
    println!("Email: {}", email.address());

    print!("Enter the email: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let email = Tempmail::parse(&input.trim());
    let otp = email.get_otp(EMAIL, OTP, OTP_LEN).await?;
    println!("OTP: {}", otp);

    Ok(())
}
