use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::utils::deserialize_time;

#[derive(Deserialize)]
pub(super) struct _ClassLogin {
    pub result: _ClassLoginResult,
}

#[derive(Deserialize)]
pub(super) struct _ClassLoginResult {
    pub id: String,
}

#[derive(Deserialize)]
pub(super) struct _ClassCourses {
    #[serde(deserialize_with = "deserialize_filtered_courses")]
    pub result: Vec<ClassCourse>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClassCourse {
    #[serde(rename = "course_id")]
    pub id: String,
    #[serde(rename = "course_name")]
    pub name: String,
    #[serde(rename = "teacher_name")]
    pub teacher: String,
}

fn deserialize_filtered_courses<'de, D>(deserializer: D) -> Result<Vec<ClassCourse>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut courses: Vec<ClassCourse> = Vec::new();
    let values: Vec<Value> = Deserialize::deserialize(deserializer)?;

    for value in values {
        if let Ok(course) = serde_json::from_value::<ClassCourse>(value.clone()) {
            if !course.teacher.is_empty() {
                courses.push(course);
            }
        }
    }

    Ok(courses)
}

#[derive(Deserialize)]
pub(super) struct _ClassSchedules {
    pub result: Vec<ClassSchedule>,
}

#[derive(Debug, Deserialize)]
pub struct ClassSchedule {
    #[serde(rename = "courseSchedId")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(rename = "classBeginTime")]
    pub time: time::PrimitiveDateTime,
    #[serde(rename = "signStatus")]
    pub state: String,
}
