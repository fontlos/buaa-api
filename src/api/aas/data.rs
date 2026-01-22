use serde::{Deserialize, Deserializer};
use time::Weekday;

use crate::error::Error;

#[derive(Debug, Deserialize)]
pub(super) struct Res<T> {
    datas: T,
    code: String,
    msg: Option<String>,
}

impl<'de, T: Deserialize<'de>> Res<T> {
    pub(crate) fn parse(v: &'de [u8], err: &'static str) -> crate::Result<T> {
        let res: Res<T> = serde_json::from_slice(&v)?;
        if res.code == "0" {
            Ok(res.datas)
        } else {
            let source = format!(
                "Code: {}, Message: {}",
                res.code,
                res.msg.unwrap_or("Unknown error".into())
            );
            Err(Error::server(err).with_label("Aas").with_source(source))
        }
    }
}

// 辅助容器
pub(super) struct Data<T>(pub T);

// ====================
// 用于 get_config
// ====================

/// School calendar config
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Current week
    #[serde(rename = "classWeek")]
    pub week: u8,
    // 为什么纯数字的是学期名称, 另一个汉字的是学期代码, 什么**命名
    /// Current Term
    #[serde(rename = "xnxqmc")]
    pub term: String,
}

impl<'de> Deserialize<'de> for Data<Config> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            #[serde(rename = "welcomeInfo")]
            info: Config,
        }
        let i = I::deserialize(deserializer)?;
        Ok(Data(i.info))
    }
}

/// Term schedule
#[derive(Debug, Deserialize)]
pub struct Schedules {
    /// Scheduled (offline) schedule (Arranged time and position)
    #[serde(rename = "arrangedList")]
    pub scheduled: Vec<Schedule>,
    /// Unscheduled (online) schedule (Not arranged time and position)
    #[serde(rename = "notArrangeList")]
    pub unscheduled: Vec<ScheduleInfo>,
}

/// Schedule base info. For both offline schedule and online schedule
#[derive(Debug, Deserialize)]
pub struct ScheduleInfo {
    /// Course ID. Format 'B[xxxxxxxxx]'
    #[serde(rename = "courseCode")]
    pub course_id: String,
    /// Class ID. Format '[yyyy][yyyy][t]B[xxxxxxxxx][zzz]'
    #[serde(rename = "teachClassId")]
    pub class_id: String,
    /// Course name
    #[serde(rename = "courseName")]
    pub name: String,
    /// Teacher and teach weeks
    #[serde(rename = "weeksAndTeachers")]
    pub teacher: String,
    /// Credit
    #[serde(deserialize_with = "deserialize_credit")]
    pub credit: f32,
}

fn deserialize_credit<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    value.parse::<f32>().map_err(serde::de::Error::custom)
}

/// Course schedule. Only for offline schedule
#[derive(Debug, Deserialize)]
pub struct Schedule {
    /// Schedule base info
    #[serde(flatten)]
    pub info: ScheduleInfo,
    /// Weekday
    #[serde(deserialize_with = "deserialize_weekday")]
    #[serde(rename = "dayOfWeek")]
    pub weekday: Weekday,
    /// Begin time
    #[serde(rename = "beginTime")]
    pub begin_time: String,
    /// End time
    #[serde(rename = "endTime")]
    pub end_time: String,
    /// Begin slots
    #[serde(rename = "beginSection")]
    pub begin_slot: u8,
    /// End slots
    #[serde(rename = "endSection")]
    pub end_slot: u8,
    /// Class position
    #[serde(rename = "placeName")]
    pub position: Option<String>,
}

fn deserialize_weekday<'de, D>(deserializer: D) -> Result<Weekday, D::Error>
where
    D: Deserializer<'de>,
{
    let value: u8 = Deserialize::deserialize(deserializer)?;
    match value {
        1 => Ok(Weekday::Monday),
        2 => Ok(Weekday::Tuesday),
        3 => Ok(Weekday::Wednesday),
        4 => Ok(Weekday::Thursday),
        5 => Ok(Weekday::Friday),
        6 => Ok(Weekday::Saturday),
        7 => Ok(Weekday::Sunday),
        _ => Err(serde::de::Error::custom("Unexpected value")),
    }
}
