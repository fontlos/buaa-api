use serde::{Deserialize, Deserializer};
use serde_json::Value;
use time::{PrimitiveDateTime, Weekday, format_description};

#[derive(Deserialize)]
pub(super) struct _SpocRes1 {
    pub content: SpocWeek,
}

/// For `get_week_schedule`, you can get it through `get_week`, and manual builds are generally not recommended
#[derive(Debug, Deserialize)]
pub struct SpocWeek {
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(rename = "pjmrrq")]
    pub time: (String, String),
    #[serde(rename = "mrxq")]
    pub term: String,
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<(String, String), D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let mut s = s.split(",");
    s.next();
    Ok((s.next().unwrap().to_string(), s.next().unwrap().to_string()))
}

#[derive(Deserialize)]
pub(super) struct _SpocRes2 {
    pub content: Vec<SpocSchedule>,
}

#[derive(Debug, Deserialize)]
pub struct SpocSchedule {
    #[serde(deserialize_with = "deserialize_spoc_day")]
    pub weekday: Weekday,
    #[serde(rename = "skdd")]
    pub position: String,
    #[serde(rename = "jsxm")]
    pub teacher: String,
    #[serde(rename = "kcmc")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_time_range")]
    #[serde(rename = "kcsj")]
    pub time: SpocTimeRange,
}

fn deserialize_spoc_day<'de, D>(deserializer: D) -> Result<Weekday, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    match value.as_str() {
        Some("monday") => Ok(Weekday::Monday),
        Some("tuesday") => Ok(Weekday::Tuesday),
        Some("wednesday") => Ok(Weekday::Wednesday),
        Some("thursday") => Ok(Weekday::Thursday),
        Some("friday") => Ok(Weekday::Friday),
        Some("saturday") => Ok(Weekday::Saturday),
        Some("sunday") => Ok(Weekday::Sunday),
        _ => Err(serde::de::Error::custom("Unexpected value")),
    }
}

#[derive(Debug)]
pub struct SpocTimeRange {
    pub start: PrimitiveDateTime,
    pub end: PrimitiveDateTime,
}

fn deserialize_time_range<'de, D>(deserializer: D) -> Result<SpocTimeRange, D::Error>
where
    D: Deserializer<'de>,
{
    let format_string = format_description::parse("[year]-[month]-[day] [hour]:[minute]").unwrap();

    let s: String = Deserialize::deserialize(deserializer)?;

    let parts: Vec<&str> = s.split(' ').collect();
    if parts.len() != 2 {
        return Err(serde::de::Error::custom("Invalid time range format"));
    }

    let date_part = parts[0];
    let time_parts: Vec<&str> = parts[1].split('-').collect();
    if time_parts.len() != 2 {
        return Err(serde::de::Error::custom("Invalid time range format"));
    }

    let start_time = format!("{} {}", date_part, time_parts[0]);
    let end_time = format!("{} {}", date_part, time_parts[1]);

    let start =
        PrimitiveDateTime::parse(&start_time, &format_string).map_err(serde::de::Error::custom)?;
    let end =
        PrimitiveDateTime::parse(&end_time, &format_string).map_err(serde::de::Error::custom)?;

    Ok(SpocTimeRange { start, end })
}
