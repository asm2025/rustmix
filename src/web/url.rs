use std::{borrow::Cow, string::FromUtf8Error};
pub use url::{ParseError, Url};
use urlencoding::{decode, encode};

use crate::string::StringEx;

pub fn url_encode(value: &str) -> Cow<str> {
    encode(value)
}

pub fn url_decode(value: &str) -> Result<Cow<str>, FromUtf8Error> {
    decode(value)
}

pub fn create(value: &str) -> Result<Url, ParseError> {
    let url = value.trim_many(&['/', ' ']);

    if url.is_empty() {
        return Err(ParseError::EmptyHost);
    }

    match Url::parse(&url) {
        Ok(url) => Ok(url),
        Err(ParseError::RelativeUrlWithoutBase) => {
            let url = format!("http://localhost/{}", url);
            Url::parse(&url)
        }
        Err(e) => Err(e),
    }
}

pub fn join(base: &Url, path: &str) -> Result<Url, ParseError> {
    if path.is_empty() {
        return Ok(base.to_owned());
    }

    let url = base.clone();
    url.join(path)
}

pub fn remove(url: &mut Url, path: &str) {
    if path.is_empty() {
        return;
    }
    url.set_path(&url.path().replace(path, ""));
}
