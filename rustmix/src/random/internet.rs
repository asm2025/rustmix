use fake::{
    faker::{filesystem::raw as f_filesystem, http::raw as f_http, internet::raw as f_internet},
    locales, Fake,
};
use fake_user_agent;
use fake_useragent::UserAgentsBuilder;
pub use fake_useragent::{Browsers, UserAgents};
use std::ops::Range;

pub fn status_code() -> String {
    f_http::RfcStatusCode(locales::EN).fake()
}

pub fn valid_status_code() -> String {
    f_http::ValidStatusCode(locales::EN).fake()
}

pub fn mime_type() -> String {
    f_filesystem::MimeType(locales::EN).fake()
}

pub fn free_email() -> String {
    f_internet::FreeEmail(locales::EN).fake()
}

pub fn safe_email() -> String {
    f_internet::SafeEmail(locales::EN).fake()
}

pub fn free_email_provider() -> String {
    f_internet::FreeEmailProvider(locales::EN).fake()
}

pub fn domain_suffix() -> String {
    f_internet::DomainSuffix(locales::EN).fake()
}

pub fn username() -> String {
    f_internet::Username(locales::EN).fake()
}

pub fn password(r: Range<usize>) -> String {
    f_internet::Password(locales::EN, r).fake()
}

pub fn ipv4() -> String {
    f_internet::IPv4(locales::EN).fake()
}

pub fn ipv6() -> String {
    f_internet::IPv6(locales::EN).fake()
}

pub fn ip() -> String {
    f_internet::IP(locales::EN).fake()
}

pub fn mac_address() -> String {
    f_internet::MACAddress(locales::EN).fake()
}

pub fn user_agent() -> String {
    fake_user_agent::get_rua().to_owned()
}

pub fn chrome_user_agent() -> String {
    fake_user_agent::get_chrome_rua().to_owned()
}

pub fn edge_user_agent() -> String {
    fake_user_agent::get_edge_rua().to_owned()
}

pub fn firefox_user_agent() -> String {
    fake_user_agent::get_firefox_rua().to_owned()
}

pub fn opera_user_agent() -> String {
    fake_user_agent::get_opera_rua().to_owned()
}

pub fn safari_user_agent() -> String {
    fake_user_agent::get_safari_rua().to_owned()
}

pub fn ie_user_agent() -> String {
    fake_user_agent::get_ie_rua().to_owned()
}

pub fn build_user_agents(browsers: Option<Browsers>) -> UserAgents {
    UserAgentsBuilder::new()
        .cache(true)
        .thread(2)
        .set_browsers(browsers.unwrap_or(Browsers::new()))
        .build()
}
