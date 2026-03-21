use serde::{Deserialize, Deserializer, Serialize};

// 经典司马接口每个响应体都不一样, 直接解析
// {"code":0,"msg":"success","list":{OBJ,OBJ}}
// {"code":10000,"message":"操作成功","data":{}}
// {"success":true,"result":{"code":200,"msg":"", list:[]}}
use crate::api::Data;

/// Request Body Payload
pub enum Payload<'a, P: Serialize + ?Sized> {
    /// Query data
    Query(&'a P),
    /// JSON data
    Json(&'a P),
    /// No data
    Empty,
}

/// Schedule info
#[derive(Debug, Clone, Deserialize)]
pub struct Schedule {
    /// Course ID
    pub course_id: String,
    /// Live ID
    #[serde(rename = "id")]
    pub live_id: String,
    /// Course name
    #[serde(rename = "course_title")]
    pub name: String,
    /// Teacher name
    #[serde(rename = "teacher_name")]
    pub teacher: String,
}

impl<'de> Deserialize<'de> for Data<[Vec<Schedule>; 7]> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            result: J,
        }
        #[derive(Deserialize)]
        struct J {
            list: [K; 7],
        }
        #[derive(Deserialize)]
        struct K {
            course: Vec<Schedule>,
        }
        let i = I::deserialize(deserializer)?;
        Ok(Data(i.result.list.map(|k| k.course)))
    }
}
