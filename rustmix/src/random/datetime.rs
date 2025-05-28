use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use fake::{faker::chrono::raw as f_chrono, locales, Fake};

pub fn naive() -> NaiveDateTime {
    f_chrono::DateTime(locales::EN).fake()
}

pub fn str() -> String {
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

pub fn between(date1: DateTime<Utc>, date2: DateTime<Utc>) -> DateTime<Utc> {
    f_chrono::DateTimeBetween(locales::EN, date1, date2).fake()
}

pub fn between_str(date1: DateTime<Utc>, date2: DateTime<Utc>) -> String {
    f_chrono::DateTimeBetween(locales::EN, date1, date2).fake()
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

pub fn time_str() -> String {
    f_chrono::Time(locales::EN).fake()
}

pub fn duration() -> Duration {
    f_chrono::Duration(locales::EN).fake()
}
