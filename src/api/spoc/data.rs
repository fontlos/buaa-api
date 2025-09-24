use serde::{Deserialize, Deserializer};
use time::{PrimitiveDateTime, Weekday, format_description};

// ====================
// 用于 get_week
// ====================

// Res<Week>
/// For `get_week_schedule`, you can get it through `get_week`, and manual builds are generally not recommended
#[derive(Debug, Deserialize)]
pub struct Week {
    /// Week date range
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(rename = "pjmrrq")]
    pub date: (String, String),
    /// Term ID
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

// ====================
// 用于 get_week_schedule
// ====================

// Res<Vec<Schedule>>
/// Weekly Schedule item
#[derive(Debug, Deserialize)]
pub struct Schedule {
    /// Course weekday
    #[serde(deserialize_with = "deserialize_weekday")]
    pub weekday: Weekday,
    // 极少数课程可能为空. 那我问你, 提供个空字符串保证结构会死吗
    /// Classroom
    #[serde(default)]
    #[serde(rename = "skdd")]
    pub position: String,
    /// Teacher
    #[serde(rename = "jsxm")]
    pub teacher: String,
    /// Help ClassApi to filter today's classes
    #[serde(deserialize_with = "deserialize_class_id")]
    #[serde(rename = "bjmc")]
    pub class_id: String,
    /// Course name
    #[serde(rename = "kcmc")]
    pub name: String,
    /// Course time range
    #[serde(deserialize_with = "deserialize_time_range")]
    #[serde(rename = "kcsj")]
    pub time: TimeRange,
}

fn deserialize_class_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let class_id = s.strip_prefix("教学班:").ok_or(serde::de::Error::custom(
        "Unexpected value in SpocSchedule class_id",
    ))?;
    Ok(class_id.to_string())
}

fn deserialize_weekday<'de, D>(deserializer: D) -> Result<Weekday, D::Error>
where
    D: Deserializer<'de>,
{
    let value: String = Deserialize::deserialize(deserializer)?;
    match value.as_str() {
        "monday" => Ok(Weekday::Monday),
        "tuesday" => Ok(Weekday::Tuesday),
        "wednesday" => Ok(Weekday::Wednesday),
        "thursday" => Ok(Weekday::Thursday),
        "friday" => Ok(Weekday::Friday),
        "saturday" => Ok(Weekday::Saturday),
        "sunday" => Ok(Weekday::Sunday),
        _ => Err(serde::de::Error::custom(
            "Unexpected value in SpocSchedule weekday",
        )),
    }
}

/// Course time range
#[derive(Debug)]
pub struct TimeRange {
    /// Course start time
    pub start: PrimitiveDateTime,
    /// Course end time
    pub end: PrimitiveDateTime,
}

fn deserialize_time_range<'de, D>(deserializer: D) -> Result<TimeRange, D::Error>
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

    Ok(TimeRange { start, end })
}
