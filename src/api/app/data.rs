use serde::Deserialize;
use time::PrimitiveDateTime;

use std::collections::HashMap;

use crate::utils::deserialize_datetime;

/// Exam schedule
#[derive(Debug)]
pub struct Exams {
    /// Exam data
    pub data: Vec<Exam>,
}

// Map 展开成 Vec
impl<'de> Deserialize<'de> for Exams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: HashMap<String, Vec<Exam>> = HashMap::deserialize(deserializer)?;
        // 获取 value 时丢失了原始顺序
        let mut exams = map.into_values().flatten().collect::<Vec<Exam>>();
        // 没人会有超过二十个考试的, 没必要用额外的逻辑复杂度直接展开 JSON. 直接排序即可
        exams.sort_by(|a, b| a.start.cmp(&b.start));
        Ok(Exams { data: exams })
    }
}

/// Exam information
#[derive(Debug, Clone, Deserialize)]
pub struct Exam {
    /// Exam name
    #[serde(rename = "course_name")]
    pub name: String,
    /// Exam start time
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "exame_start_time")]
    pub start: PrimitiveDateTime,
    /// Exam end time
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "exame_end_time")]
    pub end: PrimitiveDateTime,
    // kclx 课程类型 必修/选修
    // exam_type 考试类型 期中/期末
    /// Exam position
    #[serde(rename = "location")]
    pub position: String,
}
