use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use std::time::Duration;

use crate::Result;

pub const DATE_FORMAT: &str = "%Y-%m-%d";
pub const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M";
pub const DATE_TIME_LONG_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
pub const DATE_TIME_FULL_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.f";
pub const DATE_TIME_FULL_FORMAT_TZ: &str = "%Y-%m-%dT%H:%M:%S%.fZ";

pub fn parse_date_any(value: &str) -> Result<DateTime<Utc>> {
    let date = NaiveDateTime::parse_from_str(value, DATE_TIME_FORMAT)
        .or_else(|_| NaiveDateTime::parse_from_str(value, DATE_TIME_LONG_FORMAT))
        .or_else(|_| NaiveDateTime::parse_from_str(value, DATE_TIME_FULL_FORMAT))
        .or_else(|_| NaiveDateTime::parse_from_str(value, DATE_TIME_FULL_FORMAT_TZ))
        .or_else(|_| {
            NaiveDate::parse_from_str(value, DATE_FORMAT)
                .map(|d| NaiveDateTime::new(d, NaiveTime::MIN))
        })?;
    Ok(Utc.from_utc_datetime(&date))
}

pub fn parse_date(value: &str) -> Result<DateTime<Utc>> {
    let date = NaiveDateTime::parse_from_str(value, DATE_TIME_LONG_FORMAT)?;
    Ok(Utc.from_utc_datetime(&date))
}

pub fn parse_date_ftz(value: &str) -> Result<DateTime<Utc>> {
    let date = NaiveDateTime::parse_from_str(value, DATE_TIME_FULL_FORMAT_TZ)?;
    Ok(Utc.from_utc_datetime(&date))
}

pub fn utc_today() -> DateTime<Utc> {
    Utc.from_utc_datetime(&NaiveDateTime::new(Utc::now().date_naive(), NaiveTime::MIN))
}

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let microseconds = duration.subsec_micros();
    format!(
        "{:02}:{:02}:{:02}.{:05}",
        hours,
        minutes,
        seconds,
        microseconds / 10
    )
}
