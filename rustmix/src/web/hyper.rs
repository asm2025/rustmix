extern crate hyper as _hyper;

pub use _hyper::*;
pub use hyper_util::*;
pub use tokio::{net::TcpStream, task};

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
