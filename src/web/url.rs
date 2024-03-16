pub use url::{ParseError, Url};
use urlencoding::{decode, encode};

pub fn url_encode<T: AsRef<str>>(value: T) -> String {
    encode(value.as_ref()).to_string()
}

pub fn url_decode<T: AsRef<str>>(value: T) -> String {
    decode(value.as_ref()).unwrap().to_string()
}

pub fn create<T: AsRef<str>>(value: T) -> Url {
    Url::parse(value.as_ref()).unwrap()
}

pub fn join<T: AsRef<str>>(base: &Url, value: T) -> Url {
    let value = value.as_ref();

    if value.is_empty() {
        return base.to_owned();
    }

    let url = base.clone();
    url.join(value.as_ref()).unwrap()
}

pub fn remove<T: AsRef<str>>(url: &mut Url, value: T) {
    let value = value.as_ref();

    if value.is_empty() {
        return;
    }
    url.set_path(&url.path().replace(value, ""));
}
