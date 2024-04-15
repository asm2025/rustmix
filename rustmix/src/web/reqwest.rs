extern crate reqwest as _reqwest;

pub use _reqwest::*;

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

pub fn build_client() -> _reqwest::ClientBuilder {
    _reqwest::Client::builder()
        .default_headers(build_default_headers())
        .user_agent(user_agent())
        .cookie_store(true)
        .pool_max_idle_per_host(0)
        .timeout(std::time::Duration::from_secs(30))
}

pub fn build_blocking_client() -> _reqwest::blocking::ClientBuilder {
    _reqwest::blocking::Client::builder()
        .default_headers(build_default_headers())
        .user_agent(user_agent())
        .cookie_store(true)
        .pool_max_idle_per_host(0)
        .timeout(std::time::Duration::from_secs(30))
}

pub fn build_client_with_headers(headers: header::HeaderMap) -> _reqwest::ClientBuilder {
    let builder = build_client();
    builder.default_headers(headers)
}

pub fn build_blocking_client_with_headers(
    headers: header::HeaderMap,
) -> _reqwest::blocking::ClientBuilder {
    let builder = build_blocking_client();
    builder.default_headers(headers)
}

pub fn build_client_for_api() -> _reqwest::ClientBuilder {
    let builder = build_client();
    builder.default_headers(build_default_api_headers())
}

pub fn build_blocking_client_for_api() -> _reqwest::blocking::ClientBuilder {
    let builder = build_blocking_client();
    builder.default_headers(build_default_api_headers())
}
