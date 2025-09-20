use serde::{Deserialize, Deserializer, Serialize};

use crate::utils::deserialize_datatime;

/// Response wrapper for ClassApi
#[derive(Deserialize)]
pub(super) struct _ClassRes<T> {
    #[serde(rename = "STATUS")]
    pub status: String,
    #[serde(rename = "ERRMSG")]
    pub msg: Option<String>,
    pub result: Option<T>,
}

/// Course info
#[derive(Debug, Deserialize, Serialize)]
pub struct ClassCourse {
    /// Course id. Only use to query [ClassSchedule]
    #[serde(rename = "course_id")]
    pub id: String,
    // [学期号][课程代码][课程号]
    /// Unique ID. Use to filter classes
    #[serde(rename = "fz_id")]
    pub class_id: String,
    // 因为学校 ** 的服务器可能会导致这个课程的所有小班都显示在你的列表上
    /// Course name. There may be courses with the same name.
    #[serde(rename = "course_name")]
    pub name: String,
    /// Teacher name
    #[serde(rename = "teacher_name")]
    pub teacher: String,
}

/// Course's Schedule
#[derive(Debug, Deserialize)]
pub struct ClassSchedule {
    /// Schedule ID, only use to checkin
    #[serde(rename = "courseSchedId")]
    pub id: String,
    /// Checkin time
    #[serde(deserialize_with = "deserialize_datatime")]
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
        _ => Err(serde::de::Error::custom(
            "Unexpected status in ClassSchedule",
        )),
    }
}
