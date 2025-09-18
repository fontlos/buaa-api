use serde::{Deserialize, Serialize};

use crate::utils::deserialize_datatime;

#[derive(Deserialize)]
pub(super) struct _ClassRes<T> {
    #[serde(rename = "STATUS")]
    pub status: String,
    #[serde(rename = "ERRMSG")]
    pub msg: Option<String>,
    pub result: Option<T>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassCourse {
    /// For query [ClassSchedule]
    #[serde(rename = "course_id")]
    pub id: String,
    // [学期号][课程代码][课程号]
    /// Get from SpocApi to filter today's classes
    #[serde(rename = "fz_id")]
    pub class_id: String,
    #[serde(rename = "course_name")]
    pub name: String,
    #[serde(rename = "teacher_name")]
    pub teacher: String,
}

#[derive(Debug, Deserialize)]
pub struct ClassSchedule {
    #[serde(rename = "courseSchedId")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_datatime")]
    #[serde(rename = "classBeginTime")]
    pub time: time::PrimitiveDateTime,
    #[serde(rename = "signStatus")]
    pub state: String,
}
