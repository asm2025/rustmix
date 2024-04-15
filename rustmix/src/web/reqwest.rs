extern crate reqwest as xreqwest;

pub use xreqwest::*;

use crate::random::internet::user_agent;

fn build_default_headers() -> header::HeaderMap {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-cache"),
    );
    headers.insert(header::PRAGMA, header::HeaderValue::from_static("no-cache"));
    headers.insert(
        header::CONNECTION,
        header::HeaderValue::from_static("keep-alive"),
    );
    headers
}

fn build_default_api_headers() -> header::HeaderMap {
    let mut headers = build_default_headers();
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    headers
}

pub fn build_client() -> xreqwest::ClientBuilder {
    xreqwest::Client::builder()
        .default_headers(build_default_headers())
        .user_agent(user_agent())
        .cookie_store(true)
        .pool_max_idle_per_host(0)
        .timeout(std::time::Duration::from_secs(30))
}

pub fn build_blocking_client() -> xreqwest::blocking::ClientBuilder {
    xreqwest::blocking::Client::builder()
        .default_headers(build_default_headers())
        .user_agent(user_agent())
        .cookie_store(true)
        .pool_max_idle_per_host(0)
        .timeout(std::time::Duration::from_secs(30))
}

pub fn build_client_with_headers(headers: header::HeaderMap) -> xreqwest::ClientBuilder {
    let builder = build_client();
    builder.default_headers(headers)
}

pub fn build_blocking_client_with_headers(
    headers: header::HeaderMap,
) -> xreqwest::blocking::ClientBuilder {
    let builder = build_blocking_client();
    builder.default_headers(headers)
}

pub fn build_client_for_api() -> xreqwest::ClientBuilder {
    let builder = build_client();
    builder.default_headers(build_default_api_headers())
}

pub fn build_blocking_client_for_api() -> xreqwest::blocking::ClientBuilder {
    let builder = build_blocking_client();
    builder.default_headers(build_default_api_headers())
}
