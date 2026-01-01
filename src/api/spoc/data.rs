use serde::{Deserialize, Deserializer, Serialize};
use time::macros::format_description;
use time::{PrimitiveDateTime, Weekday};

/// Request Body Payload
#[derive(Debug, Serialize)]
pub enum Payload<'a, P: Serialize + ?Sized> {
    /// Query data
    Query(&'a P),
    /// JSON data
    Json(&'a P),
}

/// Response Wrapper
#[derive(Deserialize)]
pub struct Res<T> {
    code: u32,
    msg: Option<String>,
    content: T,
}

impl<'de, T: Deserialize<'de>> Res<T> {
    pub(crate) fn parse(v: &'de [u8]) -> crate::Result<T> {
        // TODO: 这种错误怎么处理
        // {"code":0,"msg":"异常错误，错误ID：d0a067d1c90f475381e5654f8847db2e","msg_en":null,"content":"d0a067d1c90f475381e5654f8847db2e"}
        let res: Res<T> = serde_json::from_slice(&v)?;
        // 凭据过期 code 也是 200, 那你这 code 有什么用啊
        if res.code != 200 {
            let source = format!(
                "Status Code: {}. Error Message: {:?}",
                res.code,
                res.msg.unwrap_or_else(|| "No message".to_string())
            );
            return Err(crate::Error::server("Operation failed")
                .with_label("Spoc")
                .with_source(source));
        }
        Ok(res.content)
    }
}

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
    let start = s
        .next()
        .ok_or_else(|| serde::de::Error::custom("Missing start date"))?;
    let end = s
        .next()
        .ok_or_else(|| serde::de::Error::custom("Missing end date"))?;
    Ok((start.to_string(), end.to_string()))
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
    pub position: Option<String>,
    /// Teacher
    #[serde(rename = "jsxm")]
    pub teacher: String,
    /// Course name
    #[serde(rename = "kcmc")]
    pub name: String,
    /// Course time range
    #[serde(deserialize_with = "deserialize_time_range")]
    #[serde(rename = "kcsj")]
    pub time: TimeRange,
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
    let format_string = format_description!("[year]-[month]-[day] [hour]:[minute]");

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

// ====================
// 用于 query_courses
// ====================

/// Course item
#[derive(Debug, Deserialize)]
pub struct Course {
    /// Course ID
    #[serde(rename = "kcid")]
    pub id: String,
    /// Course name
    #[serde(rename = "kcmc")]
    pub name: String,
    // // Tearcher name
    // #[serde(rename = "skjs")]
    // pub teacher: String,
}
