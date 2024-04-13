use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use fake::{
    faker::{
        address::raw as f_address, administrative::raw as f_administrative,
        automotive::raw as f_automotive, barcode::raw as f_barcode, boolean::raw as f_boolean,
        chrono::raw as f_chrono, color::raw as f_color, company::raw as f_company,
        creditcard::raw as f_creditcard, currency::raw as f_currency,
        filesystem::raw as f_filesystem, finance::raw as f_finance, http::raw as f_http,
        internet::raw as f_internet, job::raw as f_job, lorem::raw as f_lorem, name::raw as f_name,
        number::raw as f_number, phone_number::raw as f_phone_number, time::raw as f_time,
    },
    locales,
    uuid::*,
    Dummy, Fake, Faker,
};
use fake_useragent::UserAgentsBuilder;
use rand::{thread_rng, Rng};
use std::ops::Range;

use crate::string::SPECIAL_CHARS;

pub fn string(len: usize) -> String {
    let mut s = String::with_capacity(len);

    for _ in 0..len {
        s.push(alphanum());
    }

    s
}

pub fn alphanum() -> char {
    let c = rand::thread_rng().gen_range(0..62);

    if c < 10 {
        (char::from_digit(c, 10).unwrap() as u8 + 48) as char
    } else if c < 36 {
        (char::from_digit(c - 10, 10).unwrap() as u8 + 65) as char
    } else {
        (char::from_digit(c - 36, 10).unwrap() as u8 + 97) as char
    }
}

pub fn char() -> char {
    let mut rnd = rand::thread_rng();
    let c = rnd.gen_range(0..62);

    if c < 10 {
        (char::from_digit(c, 10).unwrap() as u8 + 48) as char
    } else if c < 36 {
        (char::from_digit(c - 10, 10).unwrap() as u8 + 65) as char
    } else if c < 62 {
        (char::from_digit(c - 36, 10).unwrap() as u8 + 97) as char
    } else {
        SPECIAL_CHARS[rnd.gen_range(0..10)]
    }
}

pub fn float() -> f64 {
    let mut rng = thread_rng();
    rng.gen_range(0.0..1.0)
}

pub fn numeric<T>(min: T, max: T) -> T
where
    T: rand::distributions::uniform::SampleUniform + PartialOrd,
{
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

pub fn boolean() -> bool {
    f_boolean::Boolean(locales::EN, u8::MAX / 2).fake()
}

pub mod address {
    use super::*;

    pub fn building() -> String {
        f_address::BuildingNumber(locales::EN).fake()
    }

    pub fn street() -> String {
        f_address::StreetName(locales::EN).fake()
    }

    pub fn street_suffix() -> String {
        f_address::StreetSuffix(locales::EN).fake()
    }

    pub fn secondary_address() -> String {
        f_address::SecondaryAddress(locales::EN).fake()
    }

    pub fn secondary_address_type() -> String {
        f_address::SecondaryAddressType(locales::EN).fake()
    }

    pub fn city_prefix() -> String {
        f_address::CityPrefix(locales::EN).fake()
    }

    pub fn city_suffix() -> String {
        f_address::CitySuffix(locales::EN).fake()
    }

    pub fn city() -> String {
        f_address::CityName(locales::EN).fake()
    }

    pub fn state() -> String {
        f_address::StateName(locales::EN).fake()
    }

    pub fn state_abbr() -> String {
        f_address::StateAbbr(locales::EN).fake()
    }

    pub fn country() -> String {
        f_address::CountryName(locales::EN).fake()
    }

    pub fn country_code() -> String {
        f_address::CountryCode(locales::EN).fake()
    }

    pub fn zipcode() -> String {
        f_address::ZipCode(locales::EN).fake()
    }

    pub fn postalcode() -> String {
        f_address::PostCode(locales::EN).fake()
    }

    pub fn timezone() -> String {
        f_address::TimeZone(locales::EN).fake()
    }

    pub fn latitude() -> String {
        f_address::Latitude(locales::EN).fake()
    }

    pub fn longitude() -> String {
        f_address::Longitude(locales::EN).fake()
    }

    pub fn geohash(percision: u8) -> String {
        f_address::Geohash(locales::EN, percision).fake()
    }
}

pub mod administrative {
    use super::*;

    pub fn insurance_code() -> String {
        f_administrative::HealthInsuranceCode(locales::FR_FR).fake()
    }
}

pub mod automotive {
    use super::*;

    pub fn license_number() -> String {
        f_automotive::LicencePlate(locales::FR_FR).fake()
    }
}

pub mod barcode {
    use super::*;

    pub fn isbn() -> String {
        f_barcode::Isbn(locales::EN).fake()
    }

    pub fn isbn10() -> String {
        f_barcode::Isbn10(locales::EN).fake()
    }

    pub fn isbn13() -> String {
        f_barcode::Isbn13(locales::EN).fake()
    }
}

pub mod datetime {
    use fake::faker::chrono::raw::DateTimeBetween;

    use super::*;

    pub fn naive() -> NaiveDateTime {
        f_chrono::DateTime(locales::EN).fake()
    }

    pub fn str() -> String {
        f_chrono::DateTime(locales::EN).fake()
    }

    pub fn tz<T: TimeZone + Dummy<Faker>>() -> DateTime<T> {
        f_chrono::DateTime(locales::EN).fake()
    }

    pub fn before(date: DateTime<Utc>) -> DateTime<Utc> {
        f_chrono::DateTimeBefore(locales::EN, date).fake()
    }

    pub fn before_str(date: DateTime<Utc>) -> String {
        f_chrono::DateTimeBefore(locales::EN, date).fake()
    }

    pub fn after(date: DateTime<Utc>) -> DateTime<Utc> {
        f_chrono::DateTimeAfter(locales::EN, date).fake()
    }

    pub fn after_str(date: DateTime<Utc>) -> String {
        f_chrono::DateTimeAfter(locales::EN, date).fake()
    }

    pub fn between(r: DateTimeBetween<Utc>) -> DateTime<Utc> {
        f_chrono::DateTimeBetween(locales::EN, r.1, r.2).fake()
    }

    pub fn between_str(r: DateTimeBetween<Utc>) -> String {
        f_chrono::DateTimeBetween(locales::EN, r.1, r.2).fake()
    }

    pub fn date() -> NaiveDate {
        f_chrono::Date(locales::EN).fake()
    }

    pub fn date_str() -> String {
        f_chrono::Date(locales::EN).fake()
    }

    pub fn time() -> NaiveTime {
        f_chrono::Time(locales::EN).fake()
    }

    pub fn duration() -> Duration {
        f_chrono::Duration(locales::EN).fake()
    }
}

pub mod color {
    use super::*;

    pub fn name() -> String {
        f_color::Color(locales::EN).fake()
    }

    pub fn hex() -> String {
        f_color::HexColor(locales::EN).fake()
    }

    pub fn rgb() -> String {
        f_color::RgbColor(locales::EN).fake()
    }

    pub fn rgba() -> String {
        f_color::RgbaColor(locales::EN).fake()
    }

    pub fn hsl() -> String {
        f_color::HslColor(locales::EN).fake()
    }

    pub fn hsla() -> String {
        f_color::HslaColor(locales::EN).fake()
    }
}

pub mod company {
    use super::*;

    pub fn name() -> String {
        f_company::CompanyName(locales::EN).fake()
    }

    pub fn suffix() -> String {
        f_company::CompanySuffix(locales::EN).fake()
    }

    pub fn industry() -> String {
        f_company::Industry(locales::EN).fake()
    }

    pub fn catch_phase() -> String {
        f_company::CatchPhase(locales::EN).fake()
    }

    pub fn buzzword() -> String {
        f_company::Buzzword(locales::EN).fake()
    }

    pub fn buzzword_mid() -> String {
        f_company::BuzzwordMiddle(locales::EN).fake()
    }

    pub fn buzzword_tail() -> String {
        f_company::BuzzwordTail(locales::EN).fake()
    }

    pub fn bs() -> String {
        f_company::Bs(locales::EN).fake()
    }

    pub fn bs_adj() -> String {
        f_company::BsAdj(locales::EN).fake()
    }

    pub fn bs_noun() -> String {
        f_company::BsNoun(locales::EN).fake()
    }

    pub fn bs_verb() -> String {
        f_company::BsVerb(locales::EN).fake()
    }

    pub fn profession() -> String {
        f_company::Profession(locales::EN).fake()
    }
}

pub mod credit_card {
    use super::*;

    pub fn number() -> String {
        f_creditcard::CreditCardNumber(locales::EN).fake()
    }
}

pub mod currency {
    use super::*;

    pub fn code() -> String {
        f_currency::CurrencyCode(locales::EN).fake()
    }

    pub fn name() -> String {
        f_currency::CurrencyName(locales::EN).fake()
    }

    pub fn symbol() -> String {
        f_currency::CurrencySymbol(locales::EN).fake()
    }
}

pub mod filesystem {
    use super::*;

    pub fn dir_path() -> String {
        f_filesystem::DirPath(locales::EN).fake()
    }

    pub fn file_path() -> String {
        f_filesystem::FilePath(locales::EN).fake()
    }

    pub fn file_name() -> String {
        f_filesystem::FileName(locales::EN).fake()
    }

    pub fn file_extension() -> String {
        f_filesystem::FileExtension(locales::EN).fake()
    }

    pub fn mime_type() -> String {
        f_filesystem::MimeType(locales::EN).fake()
    }
}

pub mod finance {
    use super::*;

    pub fn bic() -> String {
        f_finance::Bic(locales::EN).fake()
    }

    pub fn isin() -> String {
        f_finance::Isin(locales::EN).fake()
    }
}

pub mod http {
    use super::*;

    pub fn status_code() -> String {
        f_http::RfcStatusCode(locales::EN).fake()
    }

    pub fn valid_status_code() -> String {
        f_http::ValidStatusCode(locales::EN).fake()
    }
}

pub mod internet {
    use super::*;
    pub use fake_useragent::{Browsers, UserAgents};

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

    pub fn setup_user_agents(cache_key: &str, browsers: Option<Browsers>) -> UserAgents {
        let cache_key = if cache_key.is_empty() {
			"default"
		} else {
			cache_key
		};
		let user_agents = UserAgentsBuilder::new()
            .cache(false)
			.set_browsers(browsers.unwrap_or(Browsers::all()))
            .build();
    }
}

pub mod lorem {
    use super::*;

    pub fn word() -> String {
        f_lorem::Word(locales::EN).fake()
    }

    pub fn words(count: Range<usize>) -> Vec<String> {
        f_lorem::Words(locales::EN, count).fake()
    }

    pub fn sentence(count: Range<usize>) -> String {
        f_lorem::Sentence(locales::EN, count).fake()
    }

    pub fn sentences(count: Range<usize>) -> Vec<String> {
        f_lorem::Sentences(locales::EN, count).fake()
    }

    pub fn paragraph(count: Range<usize>) -> String {
        f_lorem::Paragraph(locales::EN, count).fake()
    }

    pub fn paragraphs(count: Range<usize>) -> Vec<String> {
        f_lorem::Paragraphs(locales::EN, count).fake()
    }
}
