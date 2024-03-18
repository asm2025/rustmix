use chrono::{NaiveDateTime, NaiveTime, Utc};
use html_entities::decode_html_entities;
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;

use super::super::web::build_client;

const BASE_URL: &str = "https://email-fake.com/";

static __HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| build_client().build().unwrap());
static USERNAME_DOMAIN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
		r#"(?m)(?s)onchange="change_username\(\)".+?value="(.+?)".+? value="(.+?)" id="domainName2""#,
	)
	.unwrap()
});
static EMAIL_LINKS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?m)(?s)<a href="(.+?)".+?<div.+?>(.+?)</div>.+?time_div.+?>(.+?)</div>"#)
        .unwrap()
});
static EMAIL_FROM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?m)(?s)<div class="fem from.+?>(.+?)</div.+?<div class="fem time.+?>(.+?)</div"#)
        .unwrap()
});
static EMAIL_BODY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?m)(?s)class="elementToProof".+?>\s*(.+?)</div"#).unwrap());

pub struct EmailFake {
    username: String,
    domain: String,
}

impl EmailFake {
    pub fn new(username: &str, domain: &str) -> Self {
        if username.is_empty() {
            panic!("username is empty");
        }

        if domain.is_empty() {
            panic!("domain is empty");
        }

        EmailFake {
            username: username.to_string(),
            domain: domain.to_string(),
        }
    }

    pub async fn random() -> Result<Self, Box<dyn Error>> {
        let body = get_content(BASE_URL).await?;

        if body.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Failed to get email-fake.com",
            )));
        }

        let start = match body.find("fem coserch") {
            Some(index) => index,
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to find coserch",
                )))
            }
        };
        let body = &body[start..];
        let end = match body.find("fem dropselect") {
            Some(index) => index,
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to find dropselect",
                )))
            }
        };
        let body = &body[..end];
        let captures = match USERNAME_DOMAIN_REGEX.captures(&body) {
            Some(captures) => captures,
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Failed to find username and domain",
                )))
            }
        };
        let username = captures.get(1).unwrap().as_str();
        let domain = captures.get(2).unwrap().as_str();
        Ok(EmailFake {
            username: username.to_string(),
            domain: domain.to_string(),
        })
    }

    pub fn from(email: &EmailFake) -> Self {
        EmailFake {
            username: email.username.clone(),
            domain: email.domain.clone(),
        }
    }

    pub fn parse(email: &str) -> Self {
        if email.is_empty() {
            panic!("email is empty");
        }

        if let Some(index) = email.find('@') {
            let (username, domain) = email.split_at(index);
            EmailFake {
                username: username.to_string(),
                domain: domain[1..].to_string(),
            }
        } else {
            panic!("email is invalid");
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn address(&self) -> String {
        format!("{}@{}", self.username, self.domain)
    }

    pub async fn find_string(
        &self,
        from: Option<&str>,
        date: Option<NaiveDateTime>,
        expected: &str,
        size: usize,
    ) -> Result<String, Box<dyn Error>> {
        if expected.is_empty() {
            panic!("Expected is empty");
        }

        if size == 0 {
            panic!("Size is zero");
        }

        let mut content = get_content(&format!("{}{}", BASE_URL, self.address())).await?;
        let mut body = get_email_table(&content);

        if body.is_empty() {
            return Ok("".to_string());
        }

        let from = match from {
            Some(from) => from.to_lowercase(),
            None => "".to_owned(),
        };
        let date_min =
            date.unwrap_or_else(|| NaiveDateTime::new(Utc::now().date_naive(), NaiveTime::MIN));
        let links: Vec<(String, String, NaiveDateTime)> = EMAIL_LINKS_REGEX
            .captures_iter(&body)
            .map(|c| {
                (
                    c.get(1).unwrap().as_str().to_string(),
                    c.get(2).unwrap().as_str().to_lowercase(),
                    NaiveDateTime::parse_from_str(c.get(3).unwrap().as_str(), "%Y-%m-%d %H:%M:%S")
                        .unwrap(),
                )
            })
            .collect();

        if !links.is_empty() {
            let target = match links.iter().find(|(_, f, d)| {
                (from.is_empty() || f.to_lowercase().contains(&from)) && d >= &date_min
            }) {
                Some(item) => item.0.to_owned(),
                None => return Ok("".to_string()),
            };
            content = get_content(&format!("{}{}", BASE_URL, target)).await?;
            body = get_email_table(&content);

            if body.is_empty() {
                return Ok("".to_string());
            }
        } else if !from.is_empty() {
            if let Some(link) = EMAIL_FROM_REGEX.captures(&body) {
                let f = link.get(1).unwrap().as_str().to_lowercase();
                let d = NaiveDateTime::parse_from_str(
                    link.get(2).unwrap().as_str(),
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap();

                if !f.contains(&from) || d < date_min {
                    return Ok("".to_string());
                }
            } else {
                return Ok("".to_string());
            }
        }

        let body = match decode_html_entities(&body) {
            Ok(text) => text,
            Err(_) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to decode html entities",
            )))?,
        };

        if let Some(text) = EMAIL_BODY_REGEX.captures(&body) {
            let text = text.get(1).unwrap().as_str();

            if let Some(index) = text.find(expected) {
                let index = index + expected.len();

                let size = if size + index < text.len() {
                    size
                } else {
                    text.len() - index
                };

                if size == 0 {
                    return Ok("".to_string());
                }

                let text = &text[index..];
                let text = text.chars().take(size).collect::<String>();
                return Ok(text);
            }
        }

        Ok("".to_string())
    }
}

async fn get_content(url: &str) -> Result<String, Box<dyn Error>> {
    let response = __HTTP_CLIENT.get(url).send().await?;

    if response.status() != 200 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("Failed to get email-fake.com. {}", response.status()),
        )));
    }

    Ok(response.text().await?)
}

fn get_email_table(body: &str) -> &str {
    if body.is_empty() {
        return "";
    }

    let start = match body.find("email-table") {
        Some(index) => index,
        None => return "",
    };
    let body = &body[start..];
    let end = match body.find(r#"<script src="https://cdn.jsdelivr.net/"#) {
        Some(index) => index,
        None => return "",
    };
    &body[..end]
}
