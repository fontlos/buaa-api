//! Self-implemented Time-related utilities.

use serde::{Deserialize, Deserializer};

use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::ops::{Add, Sub};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A simple DateTime based on Unix timestamp with
///
/// Parse and format with UTC+8 timezone, but timestamp is still UTC
///
/// Supports negative timestamps (before 1970)
///
/// Not support later - earlier to get negative Duration
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    // Unix 时间戳(秒), 负数表示 1970 年之前
    timestamp: i64,
    // 纳秒部分 [0, 999,999,999]
    nanos: u32,
}

impl DateTime {
    /// Get the current DateTime(with UTC+8) since UNIX_EPOCH
    pub fn now() -> Self {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| Self {
                timestamp: d.as_secs() as i64,
                nanos: d.subsec_nanos(),
            })
            .expect("Timestamp should always get successfully")
    }

    /// Create a DateTime from a timestamp in seconds since UNIX_EPOCH
    pub fn from_timestamp(secs: i64) -> Self {
        Self {
            timestamp: secs,
            nanos: 0,
        }
    }

    /// Create a DateTime from calendar(with UTC+8) date and time components
    pub fn from_calendar(
        year: i32,
        month: Month,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Result<Self, &'static str> {
        let month = month as u32;
        // 验证日期合法性
        if !is_valid_date(year, month, day) {
            return Err("Invalid date");
        }
        if hour >= 24 || minute >= 60 || second >= 60 {
            return Err("Invalid time");
        }
        let timestamp = date_to_timestamp(year, month, day, hour, minute, second);
        Ok(Self {
            timestamp,
            nanos: 0,
        })
    }

    /// Parse with standard format: "YYYY-MM-DD HH:MM[:SS]" (with UTC+8)
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 {
            return Err("Need Date and Time separated by space");
        }

        let Date { year, month, day } = Date::parse(parts[0])?;
        #[rustfmt::skip]
        let Time { hour, minute, second } = Time::parse(parts[1])?;

        let timestamp = date_to_timestamp(year, month as u32, day, hour, minute, second);

        Ok(Self {
            timestamp,
            nanos: 0,
        })
    }

    /// Get the timestamp in seconds since UNIX_EPOCH (UTC)
    ///
    /// Negative value means before 1970-01-01 00:00:00 UTC
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Get the nanosecond part [0, 999,999,999]
    pub fn subsec_nanos(&self) -> u32 {
        self.nanos
    }

    /// Get the date part (with UTC+8)
    pub fn date(&self) -> Date {
        let (year, month, day, _, _, _) = timestamp_to_datetime(self.timestamp);
        Date {
            year,
            // 能正确解析到 DateTime 就一定是合法日期
            month: Month::from_num(month).unwrap(),
            day,
        }
    }

    /// Get the time part (with UTC+8)
    pub fn time(&self) -> Time {
        let (_, _, _, hour, minute, second) = timestamp_to_datetime(self.timestamp);
        Time {
            hour,
            minute,
            second,
        }
    }

    /// Get the weekday (with UTC+8)
    pub fn weekday(&self) -> Weekday {
        let (year, month, day, _, _, _) = timestamp_to_datetime(self.timestamp);
        // 计算该日期距离 1970-01-01 的天数
        let days = days_from_epoch(year, month, day);
        // 这一天是周四
        let w = ((4 + days) % 7 + 7) % 7;
        let weekday_num = (w + 6) % 7 + 1;
        // 能正确解析到 DateTime 就一定是合法日期
        Weekday::from_num(weekday_num as u32).unwrap()
    }

    /// Format as "YYYY-MM-DD HH:MM:SS" (with UTC+8)
    pub fn format(&self) -> String {
        let (year, month, day, hour, minute, second) = timestamp_to_datetime(self.timestamp);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        )
    }

    /// Format as "YYYY-MM-DD HH:MM" (with UTC+8)
    pub fn format_short(&self) -> String {
        let (year, month, day, hour, minute, _) = timestamp_to_datetime(self.timestamp);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            year, month, day, hour, minute
        )
    }

    /// Get the current timestamp in seconds since UNIX_EPOCH
    pub fn secs() -> u64 {
        Self::now().timestamp as u64
    }

    /// Get the current timestamp in milliseconds since UNIX_EPOCH
    pub fn millis() -> u128 {
        let now = Self::now();
        (now.timestamp as u128) * 1000 + (now.nanos as u128) / 1_000_000
    }

    /// Get the current timestamp in nanoseconds since UNIX_EPOCH
    pub fn nanos() -> u128 {
        let now = Self::now();
        (now.timestamp as u128) * 1_000_000_000 + (now.nanos as u128)
    }
}

impl Debug for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl Add<Duration> for DateTime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let mut secs = self.timestamp + rhs.as_secs() as i64;
        let mut nanos = self.nanos + rhs.subsec_nanos();

        if nanos >= 1_000_000_000 {
            secs += 1;
            nanos -= 1_000_000_000;
        }
        Self {
            timestamp: secs,
            nanos,
        }
    }
}

impl Sub<Duration> for DateTime {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        let mut secs = self.timestamp - rhs.as_secs() as i64;
        let mut nanos = self.nanos as i64 - rhs.subsec_nanos() as i64;

        if nanos < 0 {
            secs -= 1;
            nanos += 1_000_000_000;
        }
        Self {
            timestamp: secs,
            nanos: nanos as u32,
        }
    }
}

impl Sub for DateTime {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut diff_secs = self.timestamp - rhs.timestamp;
        let mut diff_nanos = self.nanos as i64 - rhs.nanos as i64;

        if diff_nanos < 0 {
            diff_secs -= 1;
            diff_nanos += 1_000_000_000;
        }

        if diff_secs >= 0 {
            Duration::new(diff_secs as u64, diff_nanos as u32)
        } else {
            // Duration 不能为负: later - earlier
            Duration::new((-diff_secs) as u64, diff_nanos as u32)
        }
    }
}

impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DateTime {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.timestamp.cmp(&other.timestamp) {
            Ordering::Equal => self.nanos.cmp(&other.nanos),
            other => other,
        }
    }
}

// 实现反序列化
impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        DateTime::parse(s).map_err(serde::de::Error::custom)
    }
}

// ========== 辅助函数: 公历日期与时间戳转换 ==========

fn is_valid_date(year: i32, month: u32, day: u32) -> bool {
    if month < 1 || month > 12 || day < 1 {
        return false;
    }
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => return false,
    };
    day <= days_in_month
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Howard Hinnant 的 civil 日期算法
fn days_from_epoch(year: i32, month: u32, day: u32) -> i64 {
    let mut y = year as i64;
    let mut m = month as i64;
    let d = day as i64;

    if m <= 2 {
        y -= 1;
        m += 12;
    }

    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m - 3) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;

    era * 146097 + doe - 719468
}

/// Howard Hinnant 的 civil 日期算法
fn days_to_date(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;

    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };

    // 年份回补
    let year = y + if m <= 2 { 1 } else { 0 };

    (year as i32, m as u32, d as u32)
}

// 注意采用东八区
fn timestamp_to_datetime(timestamp: i64) -> (i32, u32, u32, u32, u32, u32) {
    let timestamp = timestamp + 8 * 3600; // 转换为东八区时间
    let days = if timestamp >= 0 {
        timestamp / 86400
    } else {
        // 向下取整:-1 秒应该属于 1969-12-31，而不是 1969-12-30
        (timestamp + 1) / 86400 - 1
    };

    // 始终 [0, 86399]
    let seconds_in_day = timestamp - days * 86400;

    let hour = (seconds_in_day / 3600) as u32;
    let minute = ((seconds_in_day % 3600) / 60) as u32;
    let second = (seconds_in_day % 60) as u32;

    let (year, month, day) = days_to_date(days);
    (year, month, day, hour, minute, second)
}

// 注意采用东八区
fn date_to_timestamp(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> i64 {
    let days = days_from_epoch(year, month, day);
    let seconds = (hour * 3600 + minute * 60 + second) as i64;
    days * 86400 + seconds - 8 * 3600 // 转换为 UTC 时间戳
}

// ========== 其他类型 ==========

/// Date part
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    year: i32,
    month: Month,
    day: u32,
}

impl Date {
    /// Parse with standard format: "YYYY-MM-DD"
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        let date_parts: Vec<&str> = s.split('-').collect();
        if date_parts.len() != 3 {
            return Err("Date format error: need YYYY-MM-DD");
        }

        let year: i32 = date_parts[0].parse().map_err(|_| "Invalid year")?;
        let month: u32 = date_parts[1].parse().map_err(|_| "Invalid month")?;
        let day: u32 = date_parts[2].parse().map_err(|_| "Invalid day")?;

        // 验证日期合法性
        if !is_valid_date(year, month, day) {
            return Err("Invalid date");
        }

        Ok(Self {
            year,
            // 已经在上面校验过合法性了
            month: Month::from_num(month).unwrap(),
            day,
        })
    }
    /// Get year part
    pub fn year(&self) -> i32 {
        self.year
    }
    /// Get month part
    pub fn month(&self) -> Month {
        self.month
    }
    /// Get day part
    pub fn day(&self) -> u32 {
        self.day
    }
}

/// Time part
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    hour: u32,
    minute: u32,
    second: u32,
}

impl Time {
    /// Parse with standard format: "HH:MM[:SS]"
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        let time_parts: Vec<&str> = s.split(':').collect();
        if time_parts.len() < 2 || time_parts.len() > 3 {
            return Err("Time format error: need HH:MM:SS or HH:MM");
        }

        let hour: u32 = time_parts[0].parse().map_err(|_| "Invalid hour")?;
        let minute: u32 = time_parts[1].parse().map_err(|_| "Invalid minute")?;
        let second = if time_parts.len() == 3 {
            time_parts[2].parse().map_err(|_| "Invalid second")?
        } else {
            0
        };

        if hour >= 24 || minute >= 60 || second >= 60 {
            return Err("Invalid time");
        }

        Ok(Self {
            hour,
            minute,
            second,
        })
    }
    /// Get hour part
    pub fn hour(&self) -> u32 {
        self.hour
    }
    /// Get minute part
    pub fn minute(&self) -> u32 {
        self.minute
    }
    /// Get second part
    pub fn second(&self) -> u32 {
        self.second
    }
}

#[expect(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

impl Month {
    /// Convert month number (1-12) to Month
    pub fn from_num(n: u32) -> Option<Month> {
        match n {
            1 => Some(Month::January),
            2 => Some(Month::February),
            3 => Some(Month::March),
            4 => Some(Month::April),
            5 => Some(Month::May),
            6 => Some(Month::June),
            7 => Some(Month::July),
            8 => Some(Month::August),
            9 => Some(Month::September),
            10 => Some(Month::October),
            11 => Some(Month::November),
            12 => Some(Month::December),
            _ => None,
        }
    }
}

/// A week represented by its start and end `DateTime`
#[derive(Debug, Clone)]
pub struct Week {
    pub(crate) start: DateTime,
    pub(crate) end: DateTime,
}

impl Week {
    /// Get the current week (Monday to Sunday) based on the current date
    pub fn current() -> Self {
        let now = DateTime::now();
        let weekday = now.weekday();
        // 因为 Weekday 枚举是从 1 开始的, 所以要减 1 来计算偏移
        let start = now - Duration::from_secs((weekday as u64 - 1) * 86400);
        let end = start + Duration::from_secs(6 * 86400);
        Self { start, end }
    }

    /// Get the next week
    pub fn next(&self) -> Self {
        let start = self.start + Duration::from_secs(7 * 86400);
        let end = self.end + Duration::from_secs(7 * 86400);
        Self { start, end }
    }

    /// Get the previous week
    pub fn prev(&self) -> Self {
        let start = self.start - Duration::from_secs(7 * 86400);
        let end = self.end - Duration::from_secs(7 * 86400);
        Self { start, end }
    }
}

#[expect(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Weekday {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

impl Weekday {
    /// Convert weekday number (1-7) to Weekday
    pub fn from_num(n: u32) -> Option<Weekday> {
        match n {
            1 => Some(Weekday::Monday),
            2 => Some(Weekday::Tuesday),
            3 => Some(Weekday::Wednesday),
            4 => Some(Weekday::Thursday),
            5 => Some(Weekday::Friday),
            6 => Some(Weekday::Saturday),
            7 => Some(Weekday::Sunday),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now() {
        let dt = DateTime::now();
        println!("Current DateTime: {}", dt.format());

        let date = dt.date();
        let time = dt.time();
        println!(
            "Date: {}-{:?}-{}, Time: {:02}:{:02}:{:02}",
            date.year(),
            date.month(),
            date.day(),
            time.hour(),
            time.minute(),
            time.second()
        );

        let weekday = dt.weekday();
        println!("Current weekday: {:?}", weekday);

        let timestamp = DateTime::secs();
        println!("Current timestamp (secs): {}", timestamp);

        let week = Week::current();
        println!(
            "Current week: {} - {}",
            week.start.format(),
            week.end.format()
        );
    }

    #[test]
    fn test_epoch_boundary() {
        // 1970-01-01 00:00:00 = 0 秒, 但是是东八区时间, 所以应该是 1970-01-01 08:00:00
        let ts = date_to_timestamp(1970, 1, 1, 8, 0, 0);
        assert_eq!(ts, 0);
        assert_eq!(timestamp_to_datetime(0), (1970, 1, 1, 8, 0, 0));
    }

    #[test]
    fn test_timestamp() {
        let dt = DateTime::parse("2026-01-01 00:00:00").unwrap();
        assert_eq!(dt.timestamp, 1767196800);
        assert_eq!(dt.weekday(), Weekday::Thursday);
    }

    #[test]
    fn test_negative_timestamp() {
        // 1969-12-31 23:59:59 = -1 秒, 但是是东八区时间, 所以应该是 1970-01-01 07:59:59
        let ts = date_to_timestamp(1970, 1, 1, 7, 59, 59);
        assert_eq!(ts, -1);
        assert_eq!(timestamp_to_datetime(-1), (1970, 1, 1, 7, 59, 59));
    }

    #[test]
    fn test_leap_year() {
        // 2000-02-29 (闰年)
        let dt = DateTime::parse("2000-02-29 12:00:00").unwrap();
        assert_eq!(dt.format(), "2000-02-29 12:00:00");

        // 1900-02-29 (非闰年, 应解析失败)
        assert!(DateTime::parse("1900-02-29 12:00:00").is_err());
    }

    #[test]
    fn test_time_boundaries() {
        // 可选秒格式
        assert_eq!(
            DateTime::parse("2024-03-24 14:30").unwrap().format(),
            "2024-03-24 14:30:00"
        );
        // 时间进位边界
        let dt = DateTime::parse("2024-03-24 23:59:59").unwrap();
        let next = dt + std::time::Duration::from_secs(1);
        assert_eq!(next.format(), "2024-03-25 00:00:00"); // 跨天
    }

    #[test]
    fn test_arithmetic_boundaries() {
        // 跨月末
        let jan31 = DateTime::parse("2024-01-31 12:00:00").unwrap();
        let feb1 = jan31 + std::time::Duration::from_secs(24 * 3600);
        assert_eq!(feb1.format(), "2024-02-01 12:00:00");

        // 跨闰年 2 月
        let feb28_2023 = DateTime::parse("2023-02-28 12:00:00").unwrap();
        let feb28_2024 = feb28_2023 + std::time::Duration::from_secs(365 * 24 * 3600);
        assert_eq!(feb28_2024.format(), "2024-02-28 12:00:00"); // 不是 29 日

        // 跨世纪
        let dec31_2099 = DateTime::parse("2099-12-31 23:59:59").unwrap();
        let jan1_2100 = dec31_2099 + std::time::Duration::from_secs(1);
        assert_eq!(jan1_2100.format(), "2100-01-01 00:00:00");
    }

    #[test]
    fn test_roundtrip() {
        let cases = [
            "1970-01-01 00:00:00",
            "1970-01-31 23:59:59",
            "1970-02-28 23:59:59", // 平年 2 月 28 天
            "2024-02-29 23:59:59", // 闰年 2 月 29 天
            "2024-03-24 14:30:45",
            "1969-12-31 23:59:59",
            "2000-02-29 00:00:00",
            "2100-02-28 23:59:59", // 2100 不是闰年
            "2100-12-31 23:59:59",
        ];
        for s in cases {
            let dt = DateTime::parse(s).unwrap();
            assert_eq!(dt.format(), s);
        }
    }
}
