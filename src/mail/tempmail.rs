use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use html_entities::decode_html_entities;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt::{Display, Result as DisplayResult};
use tempmail::{Domain, Tempmail};

use super::super::{
    date::{parse_date, utc_today},
    numeric::random,
    string::random_string,
    web::build_client_for_api,
};

const URL_TEMP_MAIL: &str = "https://api.internal.temp-mail.io/api/v3/email/";
const URL_EMAIL_FAKE: &str = "https://email-fake.com/";
const URL_SEC_MAIL: &str = "https://www.1secmail.com/api/v1/";

static RGX_EMAIL_FAKE_GENERATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
		r#"(?m)(?s)onchange="change_username\(\)".+?value="(.+?)".+? value="(.+?)" id="domainName2""#,
	)
	.unwrap()
});
static RGX_EMAIL_FAKE_LINKS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?m)(?s)<a href=".+?<div class="fem from.+?>(.+?)</div>.+?<div class="fem subj.+?>(.+?)</div>.+?<div class="fem time.+?>(.+?)</div>"#)
        .unwrap()
});
static RGX_EMAIL_FAKE_MESSAGE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?m)(?s)<span>From:.+?<span>(.+?)<span>Subject:.+?<h1.+?>(.+?)</h1>.+?<span>Received:.+?<span>(.+?)<span.+?<div class="elementToProof".+?>(.+?)</div>"#).unwrap()
});

static __HTTP: Lazy<reqwest::Client> = Lazy::new(|| build_client_for_api().build().unwrap());

#[derive(Serialize)]
struct NewNameLength {
    #[serde(rename = "min_name_length")]
    min: usize,
    #[serde(rename = "max_name_length")]
    max: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecMailDomain {
    SecMailCom,
    SecMailOrg,
    SecMailNet,
    WwjmpCom,
    EsiixCom,
    XojxeCom,
    YoggmCom,
    IcznnCom,
    EzzttCom,
    VjuumCom,
    LaafdCom,
    TxcctCom,
}

impl Display for SecMailDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> DisplayResult {
        match self {
            SecMailDomain::SecMailCom => write!(f, "1secmail.com"),
            SecMailDomain::SecMailNet => write!(f, "1secmail.net"),
            SecMailDomain::SecMailOrg => write!(f, "1secmail.org"),
            SecMailDomain::EsiixCom => write!(f, "esiix.com"),
            SecMailDomain::EzzttCom => write!(f, "ezztt.com"),
            SecMailDomain::IcznnCom => write!(f, "icznn.com"),
            SecMailDomain::LaafdCom => write!(f, "laafd.com"),
            SecMailDomain::TxcctCom => write!(f, "txcct.com"),
            SecMailDomain::VjuumCom => write!(f, "vjuum.com"),
            SecMailDomain::WwjmpCom => write!(f, "wwjmp.com"),
            SecMailDomain::XojxeCom => write!(f, "xojxe.com"),
            SecMailDomain::YoggmCom => write!(f, "yoggm.com"),
            SecMailDomain::IcznnCom => write!(f, "icznn.com"),
            SecMailDomain::EzzttCom => write!(f, "ezztt.com"),
            SecMailDomain::VjuumCom => write!(f, "vjuum.com"),
            SecMailDomain::LaafdCom => write!(f, "laafd.com"),
            SecMailDomain::TxcctCom => write!(f, "txcct.com"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TempMailProvider {
    Tempmail,
    EmailFake,
    SecMail(SecMailDomain),
}

impl Default for TempMailProvider {
    fn default() -> Self {
        TempMailProvider::Tempmail
    }
}

#[derive(Deserialize)]
struct TempMailMessage {
    id: String,
    from: String,
    subject: String,
    created_at: String,
}

#[derive(Deserialize)]
struct SecMailMessage {
    id: String,
    from: String,
    subject: String,
    date: String,
}

#[derive(Debug, Clone)]
pub struct TempMail {
    provider: TempMailProvider,
    username: String,
    domain: String,
}

impl TempMail {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn address(&self) -> String {
        format!("{}@{}", self.username, self.domain)
    }

    pub fn new(provider: &TempMailProvider, username: &str, domain: &str) -> Self {
        if username.is_empty() {
            panic!("username is empty");
        }

        if domain.is_empty() {
            panic!("domain is empty");
        }

        TempMail {
            provider: provider.clone(),
            username: username.to_owned(),
            domain: domain.to_owned(),
        }
    }

    pub fn from(email: &TempMail) -> Self {
        TempMail {
            provider: email.provider.clone(),
            username: email.username.to_string(),
            domain: email.domain.to_string(),
        }
    }

    pub async fn generate(provider: &TempMailProvider) -> Result<Self> {
        match provider {
            TempMailProvider::Tempmail => Self::temp_mail_generate().await,
            TempMailProvider::EmailFake => Self::email_fake_generate().await,
            TempMailProvider::SecMail(domain) => Ok(Self::sec_mail_generate(domain)),
        }
    }

    async fn temp_mail_generate() -> Result<Self> {
        let url = format!("{}{}", URL_TEMP_MAIL, "new");
        let json: Value = __HTTP
            .post(&url)
            .json(&json!(NewNameLength { min: 4, max: 32 }))
            .send()
            .await?
            .json()
            .await?;
        match json {
            Value::Object(map) => {
                let email = map.get("email").unwrap().as_str().unwrap();
                Ok(Self::parse(&TempMailProvider::Tempmail, email))
            }
            _ => panic!("Invalid response"),
        }
    }

    async fn email_fake_generate() -> Result<Self> {
        let body = Self::email_fake_get_content(URL_EMAIL_FAKE).await?;

        if body.is_empty() {
            return Err(Error::msg("Failed to get email-fake.com"));
        }

        let start = match body.find("fem coserch") {
            Some(index) => index,
            None => return Err(Error::msg("Failed to find coserch")),
        };
        let body = &body[start..];
        let end = match body.find("fem dropselect") {
            Some(index) => index,
            None => return Err(Error::msg("Failed to find dropselect")),
        };
        let body = &body[..end];
        let captures = match RGX_EMAIL_FAKE_GENERATE.captures(&body) {
            Some(captures) => captures,
            None => return Err(Error::msg("Failed to find username and domain")),
        };
        let username = captures.get(1).unwrap().as_str();
        let domain = captures.get(2).unwrap().as_str();
        Ok(TempMail {
            provider: TempMailProvider::EmailFake,
            username: username.to_string(),
            domain: domain.to_string(),
        })
    }

    fn sec_mail_generate(domain: &SecMailDomain) -> Self {
        let len = (8.0 + random() * 40.0).floor() as usize;
        let username = random_string(len);
        TempMail {
            provider: TempMailProvider::SecMail(domain.clone()),
            username,
            domain: domain.to_string(),
        }
    }

    pub fn parse(provider: &TempMailProvider, email: &str) -> Self {
        if email.is_empty() {
            panic!("email is empty");
        }

        if let Some(index) = email.find('@') {
            let (username, domain) = email.split_at(index);
            TempMail {
                provider: provider.clone(),
                username: username.to_string(),
                domain: domain[1..].to_string(),
            }
        } else {
            panic!("email is invalid");
        }
    }

    pub async fn find_string(
        &self,
        from: Option<&str>,
        subject: Option<&str>,
        date: Option<DateTime<Utc>>,
        expected: &str,
        size: usize,
    ) -> Result<String> {
        if expected.is_empty() {
            panic!("Expected is empty");
        }

        if size == 0 {
            panic!("Size is zero");
        }

        match &self.provider {
            TempMailProvider::Tempmail => {
                self.temp_mail_find_string(from, subject, date, expected, size)
                    .await
            }
            TempMailProvider::EmailFake => {
                self.email_fake_find_string(from, subject, date, expected, size)
                    .await
            }
            TempMailProvider::SecMail(domain) => {
                self.sec_mail_find_string(&domain, from, subject, date, expected, size)
                    .await
            }
        }
    }

    fn extract_value(body: &str, expected: &str, size: usize) -> String {
        if body.is_empty() {
            return "".to_string();
        }

        if let Some(index) = &body.find(expected) {
            let index = index + expected.len();
            let size = if index + size < body.len() {
                size
            } else {
                body.len() - index
            };

            if size == 0 {
                return "".to_string();
            }

            let text = &body[index..];
            let text = text.chars().take(size).collect::<String>();
            return text;
        }

        "".to_string()
    }

    async fn temp_mail_find_string(
        &self,
        from: Option<&str>,
        subject: Option<&str>,
        date: Option<DateTime<Utc>>,
        expected: &str,
        size: usize,
    ) -> Result<String> {
        let from = match from {
            Some(from) => from.to_lowercase(),
            None => "".to_owned(),
        };
        let subject = match subject {
            Some(subject) => subject.to_lowercase(),
            None => "".to_owned(),
        };
        let date_min = date.unwrap_or_else(|| utc_today());
        let url = format!("{}{}{}", URL_TEMP_MAIL, self.address(), "/messages");
        let messages: Vec<TempMailMessage> = __HTTP.get(&url).send().await?.json().await?;

        if let Some(message) = messages.iter().rev().find(|e| {
            (from.is_empty() || e.from.contains(&from))
                && (subject.is_empty() || e.subject.contains(&subject))
                && parse_date(&e.created_at).unwrap() > date_min
        }) {
            let url = format!(
                "{}{}{}/message/{}",
                URL_TEMP_MAIL,
                self.address(),
                "/messages",
                message.id
            );
            let json: Value = __HTTP.get(&url).send().await?.json().await?;
            let body = json["body_text"].as_str().unwrap_or("");
            return Ok(Self::extract_value(body, expected, size));
        }

        Ok("".to_string())
    }

    async fn email_fake_find_string(
        &self,
        from: Option<&str>,
        subject: Option<&str>,
        date: Option<DateTime<Utc>>,
        expected: &str,
        size: usize,
    ) -> Result<String> {
        if expected.is_empty() {
            panic!("Expected is empty");
        }

        if size == 0 {
            panic!("Size is zero");
        }

        let mut content =
            Self::email_fake_get_content(&format!("{}{}", URL_EMAIL_FAKE, self.address())).await?;
        let mut body = Self::email_fake_get_email_table(&content);

        if body.is_empty() {
            return Ok("".to_string());
        }

        let from = match from {
            Some(from) => from.to_lowercase(),
            None => "".to_owned(),
        };
        let subject = match subject {
            Some(subject) => subject.to_lowercase(),
            None => "".to_owned(),
        };
        let date_min = date.unwrap_or_else(|| utc_today());
        let links: Vec<(String, String, String, DateTime<Utc>)> = RGX_EMAIL_FAKE_LINKS
            .captures_iter(&body)
            .map(|c| {
                (
                    c.get(1).unwrap().as_str().to_string(),
                    c.get(2).unwrap().as_str().to_lowercase(),
                    c.get(3).unwrap().as_str().to_lowercase(),
                    parse_date(c.get(4).unwrap().as_str()).unwrap(),
                )
            })
            .collect();

        if !links.is_empty() {
            let target = match links.iter().find(|(_, f, s, d)| {
                (from.is_empty() || f.to_lowercase().contains(&from))
                    && (subject.is_empty() || s.to_lowercase().contains(&subject))
                    && d >= &date_min
            }) {
                Some(item) => item.0.to_owned(),
                None => return Ok("".to_string()),
            };
            content =
                Self::email_fake_get_content(&format!("{}{}", URL_EMAIL_FAKE, target)).await?;
            body = Self::email_fake_get_email_table(&content);

            if body.is_empty() {
                return Ok("".to_string());
            }
        } else if !from.is_empty() {
            if let Some(link) = RGX_EMAIL_FAKE_FROM.captures(&body) {
                let f = link.get(1).unwrap().as_str().to_lowercase();
                let d = parse_date(link.get(2).unwrap().as_str()).unwrap();

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

        if let Some(text) = RGX_EMAIL_FAKE_BODY.captures(&body) {
            let text = text.get(1).unwrap().as_str();
            return Ok(Self::extract_value(text, expected, size));
        }

        Ok("".to_string())
    }

    async fn email_fake_get_content(url: &str) -> Result<String> {
        let response = __HTTP.get(url).send().await?;

        if response.status() != 200 {
            return Err(Error::msg(format!(
                "Failed to get email-fake.com. {}",
                response.status()
            )));
        }

        Ok(response.text().await?)
    }

    fn email_fake_get_email_table(body: &str) -> &str {
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

    async fn sec_mail_find_string(
        &self,
        domain: &SecMailDomain,
        from: Option<&str>,
        subject: Option<&str>,
        date: Option<DateTime<Utc>>,
        expected: &str,
        size: usize,
    ) -> Result<String> {
        let from = match from {
            Some(from) => from.to_lowercase(),
            None => "".to_owned(),
        };
        let subject = match subject {
            Some(subject) => subject.to_lowercase(),
            None => "".to_owned(),
        };
        let date_min = date.unwrap_or_else(|| utc_today());
        let url = format!(
            "{}?action=getMessages&login={}&domain={}",
            URL_SEC_MAIL, self.username, domain
        );
        let messages: Vec<SecMailMessage> = __HTTP.get(&url).send().await?.json().await?;

        if let Some(message) = messages.iter().rev().find(|e| {
            (from.is_empty() || e.from.contains(&from))
                && (subject.is_empty() || e.subject.contains(&subject))
                && parse_date(&e.date).unwrap() > date_min
        }) {
            let url = format!(
                "{}?action=readMessage&login={}&domain={}&id={}",
                URL_SEC_MAIL,
                self.username(),
                self.domain,
                message.id
            );
            let json: Value = __HTTP.get(&url).send().await?.json().await?;
            let body = json["html_body"].as_str().unwrap_or("");
            return Ok(Self::extract_value(body, expected, size));
        }

        Ok("".to_string())
    }
}
