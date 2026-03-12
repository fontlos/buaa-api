use serde::{Deserialize, Deserializer, Serialize};

use crate::error::Error;
use crate::utils;
use crate::utils::time::DateTime;

// 每个字段都可能缺失, 所以手动解析
/// Respond wrapper
#[derive(Deserialize)]
pub(crate) struct Res<T> {
    result: T,
}

impl<'de, T: Deserialize<'de>> Res<T> {
    pub(crate) fn parse(v: &'de [u8]) -> crate::Result<T> {
        let status = utils::parse_by_tag(&v, "\"STATUS\":\"", "\"");
        match status {
            Some("0") => Ok(serde_json::from_slice::<Res<T>>(&v)?.result),
            // 状态码为 2 表示数据为空, 通常是今日没有课程, 没有任何其他字段
            Some("2") => Err(Error::server("Empty data list").with_label("Class")),
            Some(s) => {
                let msg = utils::parse_by_tag(&v, "\"ERRMSG\":\"", "\"");
                let source = format!("Status Code: {}. Error Message: {:?}", s, msg);
                return Err(Error::server("Operation failed")
                    .with_label("Class")
                    .with_source(source));
            }
            // 错误请求可能返回 `\r\n`
            None => Err(Error::server("Empty response").with_label("Class"))
        }
    }
}

/// Schedule of some day
#[derive(Clone, Debug, Deserialize)]
pub struct Schedule {
    /// Schedule ID. Use to checkin
    pub id: String,
    /// Course ID. Use to query [CourseSchedule]
    #[serde(rename = "courseId")]
    pub course_id: String,
    /// Course name
    #[serde(rename = "courseName")]
    pub name: String,
    /// Teacher name
    #[serde(rename = "teacherName")]
    pub teacher: String,
    /// Checkin time
    #[serde(rename = "classBeginTime")]
    pub time: DateTime,
    /// Checkin status
    #[serde(deserialize_with = "deserialize_status")]
    #[serde(rename = "signStatus")]
    pub status: bool,
}

/// Course info
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Course {
    /// Course ID. Use to query [CourseSchedule]
    #[serde(rename = "course_id")]
    pub id: String,
    /// Course name. There may be courses with the same name.
    #[serde(rename = "course_name")]
    pub name: String,
    /// Teacher name
    #[serde(rename = "teacher_name")]
    pub teacher: String,
}

/// Course Schedule
#[derive(Clone, Debug, Deserialize)]
pub struct CourseSchedule {
    /// Schedule ID, only use to checkin
    #[serde(rename = "courseSchedId")]
    pub id: String,
    /// Checkin time
    #[serde(rename = "classBeginTime")]
    pub time: DateTime,
    /// Checkin status
    #[serde(deserialize_with = "deserialize_status")]
    #[serde(rename = "signStatus")]
    pub status: bool,
}

fn deserialize_status<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    match s.as_str() {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err(serde::de::Error::custom("Unexpected status in Schedule")),
    }
}
