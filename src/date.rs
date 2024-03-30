use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, NaiveTime, TimeZone, Utc};

pub const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn parse_date(date: &str) -> Result<DateTime<Utc>> {
    let date = NaiveDateTime::parse_from_str(date, DATE_FORMAT)?;
    Ok(Utc.from_utc_datetime(&date))
}

pub fn utc_today() -> DateTime<Utc> {
    Utc.from_utc_datetime(&NaiveDateTime::new(Utc::now().date_naive(), NaiveTime::MIN))
}
