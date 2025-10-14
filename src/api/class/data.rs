use serde::{Deserialize, Deserializer, Serialize};

use crate::utils::deserialize_datetime;

/// Schedule of some day
#[derive(Debug, Deserialize)]
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
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "classBeginTime")]
    pub time: time::PrimitiveDateTime,
    /// Checkin status
    #[serde(deserialize_with = "deserialize_status")]
    #[serde(rename = "signStatus")]
    pub status: u8,
}

/// Course info
#[derive(Debug, Deserialize, Serialize)]
pub struct Course {
    /// Course ID. Use to query [CourseSchedule]
    #[serde(rename = "course_id")]
    pub id: String,
    // TODO: 应该停止解析这个不稳定 ID, 使用 `query_schedule` 更准确的获取今日课表
    // [学期号][课程代码][课程号]
    /// Unique ID. Use to filter classes
    #[serde(rename = "fz_id")]
    pub class_id: String,
    /// Course name. There may be courses with the same name.
    #[serde(rename = "course_name")]
    pub name: String,
    /// Teacher name
    #[serde(rename = "teacher_name")]
    pub teacher: String,
}

/// Course Schedule
#[derive(Debug, Deserialize)]
pub struct CourseSchedule {
    /// Schedule ID, only use to checkin
    #[serde(rename = "courseSchedId")]
    pub id: String,
    /// Checkin time
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "classBeginTime")]
    pub time: time::PrimitiveDateTime,
    /// Checkin status
    #[serde(deserialize_with = "deserialize_status")]
    #[serde(rename = "signStatus")]
    pub status: u8,
}

fn deserialize_status<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    match s.as_str() {
        "1" => Ok(1),
        "0" => Ok(0),
        _ => Err(serde::de::Error::custom("Unexpected status in Schedule")),
    }
}
