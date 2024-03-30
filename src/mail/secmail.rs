use anyhow::Result;
use chrono::{NaiveDateTime, NaiveTime, Utc};
use html_entities::decode_html_entities;
use tempmail::{Domain, Tempmail};

#[derive(Debug, Clone)]
pub struct SecMail {
    email: Tempmail,
}

impl SecMail {
    pub async fn find_string(
        &self,
        from: Option<&str>,
        date: Option<NaiveDateTime>,
        expected: &str,
        size: usize,
    ) -> Result<String> {
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
}
