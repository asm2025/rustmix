use std::error::Error;
use tempmail::{Domain, Tempmail};

pub fn get_temp_mail() -> Tempmail {
    Tempmail::random()
}

pub fn get_temp_mail_from(email: &str) -> Tempmail {
    if email.is_empty() {
        panic!("email is empty");
    }

    if let Some(index) = email.find('@') {
        let (username, domain) = email.split_at(index);
        let domain = domain_from_str(&domain[1..]);
        Tempmail::new(username, Some(domain))
    } else {
        panic!("email is invalid");
    }
}

pub fn get_temp_mail_from_domain(user_name: &str, domain: Domain) -> Tempmail {
    if user_name.is_empty() {
        panic!("user_name is empty");
    }

    Tempmail::new(user_name, Some(domain))
}

pub async fn get_temp_mail_otp(
    email: &Tempmail,
    from: &str,
    expected: &str,
    size: usize,
) -> Result<String, Box<dyn Error>> {
    if expected.is_empty() {
        panic!("Expected is empty");
    }

    if from.is_empty() {
        panic!("From is empty");
    }

    if size == 0 {
        panic!("Size is zero");
    }

    let messages = email.get_messages().await?;

    if messages.is_empty() {
        return Ok("".to_string());
    }

    if let Some(message) = messages.iter().find(|m| m.from.contains(expected)) {
        let otp = match message.body.find(expected) {
            Some(index) => {
                let start = index + expected.len() + 1;
                let end = start + size;
                message.body[start..end].to_string()
            }
            None => "".to_string(),
        };
        return Ok(otp);
    }

    Ok("".to_string())
}

fn domain_from_str(s: &str) -> Domain {
    match s.to_lowercase().as_str() {
        "1secmail.com" => Domain::SecMailCom,
        "1secmail.org" => Domain::SecMailOrg,
        "1secmail.net" => Domain::SecMailNet,
        "wwjmp.com" => Domain::WwjmpCom,
        "esiix.com" => Domain::EsiixCom,
        "xojxe.com" => Domain::XojxeCom,
        "yoggm.com" => Domain::YoggmCom,
        _ => panic!("Invalid domain"),
    }
}
