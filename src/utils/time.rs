use serde::{Deserialize, Deserializer};
use time::macros::{format_description, offset};
use time::{OffsetDateTime, PrimitiveDateTime};

use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[inline]
pub fn get_timestamp() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Timestamp should always get successfully")
}

pub fn get_time_secs() -> u64 {
    get_timestamp().as_secs()
}

pub fn get_time_millis() -> u128 {
    get_timestamp().as_millis()
}

pub fn get_time_nanos() -> u128 {
    get_timestamp().as_nanos()
}

pub fn get_datetime() -> PrimitiveDateTime {
    let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
    PrimitiveDateTime::new(now.date(), now.time())
}

pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let format_string =
        format_description!("[year]-[month]-[day] [hour]:[minute][optional [:[second]]]");

    let s: &'de str = Deserialize::deserialize(deserializer)?;

    PrimitiveDateTime::parse(s, &format_string).map_err(serde::de::Error::custom)
}
