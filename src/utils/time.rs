use serde::{Deserialize, Deserializer};
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset, format_description};

use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[inline]
pub fn get_timestamp() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get timestamp")
}

pub fn get_time_secs() -> u64 {
    get_timestamp().as_secs()
}

pub fn get_time_millis() -> u128 {
    get_timestamp().as_millis()
}

pub fn get_datetime() -> PrimitiveDateTime {
    let now_utc = OffsetDateTime::now_utc();
    let local_offset = UtcOffset::from_hms(8, 0, 0).expect("Failed to create local offset");
    let now_local = now_utc.to_offset(local_offset);
    PrimitiveDateTime::new(now_local.date(), now_local.time())
}

pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let format_string = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
        .expect("Failed to parse datetime format");

    let s: String = Deserialize::deserialize(deserializer)?;

    PrimitiveDateTime::parse(&s, &format_string).map_err(serde::de::Error::custom)
}
