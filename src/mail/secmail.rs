use anyhow::Result;
use chrono::{NaiveDateTime, NaiveTime, Utc};
use html_entities::decode_html_entities;
use tempmail::{Domain, Tempmail};

pub struct SecMail {
    email: Tempmail,
}

impl SecMail {
    const DOMAINS: [&'static str; 7] = [
        "1secmail.com",
        "1secmail.org",
        "1secmail.net",
        "wwjmp.com",
        "esiix.com",
        "xojxe.com",
        "yoggm.com",
    ];

    pub fn new(user_name: &str, domain: Domain) -> Self {
        if user_name.is_empty() {
            panic!("user_name is empty");
        }

        SecMail {
            email: Tempmail::new(user_name, Some(domain)),
        }
    }

    pub fn random() -> Self {
        SecMail {
            email: Tempmail::random(),
        }
    }

    pub fn generate() -> String {
        SecMail::random().address()
    }

    pub fn from(email: &Tempmail) -> Self {
        SecMail {
            email: email.clone(),
        }
    }

    pub fn parse(email: &str) -> Self {
        if email.is_empty() {
            panic!("email is empty");
        }

        if let Some(index) = email.find('@') {
            let (username, domain) = email.split_at(index);
            let domain = from_str(&domain[1..]).unwrap_or_else(|| panic!("domain is invalid"));
            SecMail::from(&Tempmail::new(username, Some(domain)))
        } else {
            panic!("email is invalid");
        }
    }

    pub fn username(&self) -> &str {
        &self.email.username
    }

    pub fn domain(&self) -> &Domain {
        &self.email.domain
    }

    pub fn address(&self) -> String {
        format!("{}@{}", self.email.username, self.email.domain)
    }

    pub async fn find_string(
        &self,
        from: Option<&str>,
        date: Option<NaiveDateTime>,
        expected: &str,
        size: usize,
    ) -> Result<String> {
        if expected.is_empty() {
            panic!("Expected is empty");
        }

        if size == 0 {
            panic!("Size is zero");
        }

        let from = match from {
            Some(from) => from.to_lowercase(),
            None => "".to_owned(),
        };
        let date_min =
            date.unwrap_or_else(|| NaiveDateTime::new(Utc::now().date_naive(), NaiveTime::MIN));
        let messages = self.email.get_raw_messages().await?;

        if messages.is_empty() {
            return Ok("".to_string());
        }

        if let Some(raw_message) = messages.iter().rev().find(|m| {
            (from.is_empty() || m.from.to_lowercase().contains(&from))
                && m.timestamp.naive_utc() >= date_min
        }) {
            let message = self.email.expand_raw_message(raw_message).await?;
            let body = match decode_html_entities(&message.body) {
                Ok(text) => text,
                Err(_) => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to decode html entities",
                )))?,
            };
            let str = match &body.find(expected) {
                Some(index) => {
                    let index = index + expected.len();

                    let size = if size + index < body.len() {
                        size
                    } else {
                        body.len() - index
                    };

                    if size == 0 {
                        return Ok("".to_string());
                    }

                    let text = &body[index..];
                    let text = text.chars().take(size).collect::<String>();
                    return Ok(text);
                }
                None => "".to_string(),
            };
            return Ok(str);
        }

        Ok("".to_string())
    }

    pub fn get_domains() -> Vec<&'static str> {
        Self::DOMAINS.iter().copied().collect()
    }
}

fn from_str(s: &str) -> Option<Domain> {
    if s.is_empty() {
        return None;
    }

    match s.to_lowercase().as_str() {
        "1secmail.com" => Some(Domain::SecMailCom),
        "1secmail.org" => Some(Domain::SecMailOrg),
        "1secmail.net" => Some(Domain::SecMailNet),
        "wwjmp.com" => Some(Domain::WwjmpCom),
        "esiix.com" => Some(Domain::EsiixCom),
        "xojxe.com" => Some(Domain::XojxeCom),
        "yoggm.com" => Some(Domain::YoggmCom),
        _ => None,
    }
}
