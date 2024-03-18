pub mod http;
pub mod url;

use reqwest::header;
use ua_generator::ua;

pub fn get_user_agent() -> String {
    ua::spoof_ua().to_owned()
}

pub fn build_client() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .default_headers({
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
        })
        .user_agent(get_user_agent())
        .cookie_store(true)
        .pool_max_idle_per_host(0)
        .timeout(std::time::Duration::from_secs(30))
}

pub fn build_client_with_headers(headers: header::HeaderMap) -> reqwest::ClientBuilder {
    let builder = build_client();
    builder.default_headers(headers)
}

pub fn build_client_for_api() -> reqwest::ClientBuilder {
    let builder = build_client();
    builder.default_headers({
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers
    })
}
