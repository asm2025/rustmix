use chrono::{NaiveDateTime, NaiveTime, Utc};
use html_entities::decode_html_entities;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};
use std::error::Error;

use super::super::web::build_client_for_api;

const BASE_URL: &str = "https://api.internal.temp-mail.io/api/v3/email/";

static __HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| build_client_for_api().build().unwrap());

#[derive(Serialize)]
struct NewNameLengeth {
    #[serde(rename = "min_name_length")]
    min: usize,
    #[serde(rename = "max_name_length")]
    max: usize,
}

pub struct TempMail {
    username: String,
    domain: String,
}

impl TempMail {
    pub fn new(username: &str, domain: &str) -> Self {
        if username.is_empty() {
            panic!("username is empty");
        }

        if domain.is_empty() {
            panic!("domain is empty");
        }

        TempMail {
            username: username.to_string(),
            domain: domain.to_string(),
        }
    }

    pub async fn random() -> Result<Self, Box<dyn Error>> {
        let url = format!("{}{}", BASE_URL, "new");
        let json: Value = __HTTP_CLIENT
            .post(&url)
            .json(&json!(NewNameLengeth { min: 4, max: 32 }))
            .send()
            .await?
            .json()
            .await?;

        match json {
            Value::Object(map) => {
                let email = map.get("email").unwrap().as_str().unwrap();
                Ok(Self::parse(email))
            }
            _ => panic!("Invalid response"),
        }
    }

    pub fn from(email: &TempMail) -> Self {
        TempMail {
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
            TempMail {
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

        let from = match from {
            Some(from) => from.to_lowercase(),
            None => "".to_owned(),
        };
        let date_min =
            date.unwrap_or_else(|| NaiveDateTime::new(Utc::now().date_naive(), NaiveTime::MIN));
        let url = format!("{}{}{}", BASE_URL, self.address(), "/messages");
        let json: Value = __HTTP_CLIENT.get(&url).send().await?.json().await?;

        match json {
            Value::Array(messages) => {
                if messages.is_empty() {
                    return Ok("".to_string());
                }

                if let Some(raw_message) = messages.iter().rev().find(|m| {
                    (from.is_empty() || m["from"].as_str().unwrap_or("").contains(&from))
                        && NaiveDateTime::parse_from_str(
                            m["created_at"].as_str().unwrap(),
                            "%Y-%m-%dT%H:%M:%S%.fZ",
                        )
                        .unwrap()
                            > date_min
                }) {
                    let body = raw_message["body_text"].as_str().unwrap_or("");
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
                    Ok(str)
                } else {
                    Ok("".to_string())
                }
            }
            _ => panic!("Invalid response"),
        }
    }
}
