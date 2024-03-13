use serde::Deserialize;
use std::{collections::HashMap, net::IpAddr};
use url::Url;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ResponseHeaders {
    pub accept: String,
    pub host: String,
    #[serde(flatten)]
    pub dynamic: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub url: Option<Url>,
    pub origin: Option<IpAddr>,
    pub status: Option<u32>,
    pub headers: Option<ResponseHeaders>,
    pub cookies: Option<HashMap<String, String>>,
    pub args: Option<HashMap<String, String>>,
    pub data: Option<String>,
    pub form: Option<HashMap<String, String>>,
    pub files: Option<HashMap<String, String>>,
}
