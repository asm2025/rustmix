use std::error::Error;
use tempmail::{Domain, Tempmail as TempMail};

pub struct Tempmail {
    email: TempMail,
}

impl Tempmail {
    pub fn new() -> Self {
        Tempmail {
            email: TempMail::random(),
        }
    }

    pub fn from(email: TempMail) -> Self {
        Tempmail { email }
    }

    pub fn from_domain(user_name: &str, domain: Domain) -> Self {
        if user_name.is_empty() {
            panic!("user_name is empty");
        }

        Tempmail {
            email: TempMail::new(user_name, Some(domain)),
        }
    }

    pub fn parse(email: &str) -> Self {
        if email.is_empty() {
            panic!("email is empty");
        }

        if let Some(index) = email.find('@') {
            let (username, domain) = email.split_at(index);
            let domain = domain_from_str(&domain[1..]).unwrap();
            Tempmail::from(TempMail::new(username, Some(domain)))
        } else {
            panic!("email is invalid");
        }
    }

    pub fn username(&self) -> &String {
        &self.email.username
    }

    pub fn domain(&self) -> &Domain {
        &self.email.domain
    }

    pub fn address(&self) -> String {
        format!("{}@{}", self.email.username, self.email.domain)
    }

    pub fn set_email(&mut self, email: TempMail) {
        self.email = email;
    }

    pub async fn get_otp(
        &self,
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

        let messages = self.email.get_messages().await?;

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
}

fn domain_from_str(s: &str) -> Result<Domain, std::io::Error> {
    match s.to_lowercase().as_str() {
        "1secmail.com" => Ok(Domain::SecMailCom),
        "1secmail.org" => Ok(Domain::SecMailOrg),
        "1secmail.net" => Ok(Domain::SecMailNet),
        "wwjmp.com" => Ok(Domain::WwjmpCom),
        "esiix.com" => Ok(Domain::EsiixCom),
        "xojxe.com" => Ok(Domain::XojxeCom),
        "yoggm.com" => Ok(Domain::YoggmCom),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid domain",
        )),
    }
}
