use serde::{Deserialize, Deserializer};
use time::{Date, OffsetDateTime, PrimitiveDateTime, UtcOffset, format_description};

use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[inline]
pub fn get_timestamp() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

pub fn get_time_secs() -> u64 {
    get_timestamp().as_secs()
}

pub fn get_time_millis() -> u128 {
    get_timestamp().as_millis()
}

pub fn get_datatime() -> PrimitiveDateTime {
    let now_utc = OffsetDateTime::now_utc();
    let local_offset = UtcOffset::from_hms(8, 0, 0).unwrap();
    let now_local = now_utc.to_offset(local_offset);
    PrimitiveDateTime::new(now_local.date(), now_local.time())
}

pub fn parse_date(data: &str) -> Date {
    let format_string = time::format_description::parse("[year]-[month]-[day]").unwrap();
    Date::parse(data, &format_string).unwrap()
}

pub(crate) fn deserialize_datatime<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let format_string =
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

    let s: String = Deserialize::deserialize(deserializer)?;

    PrimitiveDateTime::parse(&s, &format_string).map_err(serde::de::Error::custom)
}
