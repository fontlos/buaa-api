//! Time-related utilities.

use serde::{Deserialize, Deserializer};
use time::macros::{format_description, offset};
use time::{OffsetDateTime, PrimitiveDateTime};

use std::fmt::Display;
use std::ops::Deref;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub use time::{Date, Month, Time, Weekday};

/// A wrapper of `PrimitiveDateTime`
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct DateTime(PrimitiveDateTime);

impl DateTime {
    /// Create a new `DateTime` from `Date` and `Time`
    pub fn new(date: Date, time: Time) -> Self {
        Self(PrimitiveDateTime::new(date, time))
    }

    /// Parse a `DateTime` from a string with a custom format description
    pub fn parse(s: &str, format: &str) -> crate::Result<Self> {
        let format_string = time::format_description::parse(format)
            .map_err(|e| crate::Error::parse("Bad time format description").with_source(e))?;
        PrimitiveDateTime::parse(s, &format_string)
            .map(DateTime)
            .map_err(|e| crate::Error::parse("Bad Time").with_source(e))
    }

    /// Get the current datetime with timezone offset of +8
    pub fn now() -> Self {
        let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
        Self(PrimitiveDateTime::new(now.date(), now.time()))
    }

    /// Get the current timestamp as `Duration` since UNIX_EPOCH
    #[inline]
    pub fn timestamp() -> Duration {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Timestamp should always get successfully")
    }

    /// Get the current timestamp in seconds since UNIX_EPOCH
    pub fn secs() -> u64 {
        Self::timestamp().as_secs()
    }

    /// Get the current timestamp in milliseconds since UNIX_EPOCH
    pub fn millis() -> u128 {
        Self::timestamp().as_millis()
    }

    /// Get the current timestamp in nanoseconds since UNIX_EPOCH
    pub fn nanos() -> u128 {
        Self::timestamp().as_nanos()
    }
}

// 标准日期格式
const STANDARD_FORMAT: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute][optional [:[second]]]");

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
            .format(&STANDARD_FORMAT)
            .map_err(|_| std::fmt::Error)?
            .fmt(f)
    }
}

impl Deref for DateTime {
    type Target = PrimitiveDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        PrimitiveDateTime::parse(s, &STANDARD_FORMAT)
            .map(DateTime)
            .map_err(serde::de::Error::custom)
    }
}

// 一些内部使用的工具方法
impl DateTime {
    // For SpocAPI. 解析标准格式日期字符串: YYYY-MM-DD HH:MM[:SS]
    pub(crate) fn from_standard(s: &str) -> Result<Self, &'static str> {
        PrimitiveDateTime::parse(s, &STANDARD_FORMAT)
            .map(DateTime)
            .map_err(|_| "Bad Time")
    }

    // For ClassAPI. 将日期转换为 YYYYMMDD 格式的字符串
    pub(crate) fn to_date(&self) -> String {
        let format = format_description!("[year][month][day]");
        self.format(&format).unwrap()
    }
}
