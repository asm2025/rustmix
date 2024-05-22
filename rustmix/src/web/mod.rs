#[cfg(feature = "mail")]
pub mod mail;
pub mod reqwest;

use url::{ParseError, Url};
use urlencoding::{decode, encode};

use crate::{
    web::reqwest::{blocking::Client as BlockingClient, Client},
    Result,
};

pub const REMOTE_IP_URL: &'static str = "https://api.ipify.org";

pub fn url_encode<T: AsRef<str>>(value: T) -> String {
    encode(value.as_ref()).to_string()
}

pub fn url_decode<T: AsRef<str>>(value: T) -> String {
    decode(value.as_ref()).unwrap().to_string()
}

pub fn create<T: AsRef<str>>(value: T) -> Result<Url> {
    const LOCALHOST: &str = "http://localhost";

    let value = value.as_ref();

    if value.is_empty() {
        return Ok(Url::parse(LOCALHOST)?);
    }

    match Url::parse(value) {
        Ok(it) => Ok(it.into()),
        Err(ParseError::RelativeUrlWithoutBase) => {
            Url::parse(LOCALHOST)?.join(value).map_err(Into::into)
        }
        Err(_) => Url::parse(&url_encode(value)).map_err(Into::into),
    }
}

fn append_if_not_empty<T: AsRef<str>>(base: &Url, component: T) -> Result<Url> {
    let component = component.as_ref();

    if component.is_empty() {
        return Ok(base.clone());
    }
    base.join(component).map_err(Into::into)
}

pub trait AsUrl<T> {
    fn as_url(&self) -> Result<Url>;
}

impl<T: AsRef<str>> AsUrl<T> for T {
    fn as_url(&self) -> Result<Url> {
        create(self)
    }
}

impl<T: AsRef<str>> AsUrl<T> for (T, T) {
    fn as_url(&self) -> Result<Url> {
        let base = create(&self.0)?;
        append_if_not_empty(&base, &self.1)
    }
}

impl<T: AsRef<str>> AsUrl<T> for (T, T, T) {
    fn as_url(&self) -> Result<Url> {
        let url = create(&self.0)?;
        let url = append_if_not_empty(&url, &self.1)?;
        let url = append_if_not_empty(&url, &self.2)?;
        Ok(url)
    }
}

impl<T: AsRef<str>> AsUrl<T> for (T, T, T, T) {
    fn as_url(&self) -> Result<Url> {
        let url = create(&self.0)?;
        let url = append_if_not_empty(&url, &self.1)?;
        let url = append_if_not_empty(&url, &self.2)?;
        let url = append_if_not_empty(&url, &self.3)?;
        Ok(url)
    }
}

impl<T: AsRef<str>> AsUrl<T> for (T, T, T, T, T) {
    fn as_url(&self) -> Result<Url> {
        let url = create(&self.0)?;
        let url = append_if_not_empty(&url, &self.1)?;
        let url = append_if_not_empty(&url, &self.2)?;
        let url = append_if_not_empty(&url, &self.3)?;
        let url = append_if_not_empty(&url, &self.4)?;
        Ok(url)
    }
}

impl<T: AsRef<str>, const N: usize> AsUrl<T> for [T; N] {
    fn as_url(&self) -> Result<Url> {
        self.iter().try_fold(create(&self[0])?, |url, component| {
            append_if_not_empty(&url, component)
        })
    }
}

impl<T: AsRef<str>> AsUrl<T> for Vec<T> {
    fn as_url(&self) -> Result<Url> {
        self.iter().try_fold(create(&self[0])?, |url, component| {
            append_if_not_empty(&url, component)
        })
    }
}

pub fn remove<T: AsRef<str>>(url: &mut Url, value: T) {
    let value = value.as_ref();

    if value.is_empty() {
        return;
    }
    url.set_path(&url.path().replace(value, ""));
}

pub fn get_public_ip(client: &BlockingClient) -> Result<String> {
    let response = client.get(REMOTE_IP_URL).send()?;

    if !response.status().is_success() {
        return Err(response.error_for_status().unwrap_err().into());
    }

    let text = response.text()?;
    Ok(text)
}

pub async fn get_public_ip_async(client: &Client) -> Result<String> {
    let response = client.get(REMOTE_IP_URL).send().await?;

    if !response.status().is_success() {
        return Err(response.error_for_status().unwrap_err().into());
    }

    let text = response.text().await?;
    Ok(text)
}
