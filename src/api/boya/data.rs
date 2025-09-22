use serde::{Deserialize, Deserializer};
use serde_json::Value;
use time::PrimitiveDateTime;

use crate::utils::deserialize_datatime;

// 内部辅助容器, 因为所需数据普遍在 data 字段内部的下一层包装
#[derive(Debug)]
pub(super) struct Data<T>(pub T);

// ====================
// 用于 query_courses
// ====================

// Res<Data<Vec<Course>>>
impl<'de> Deserialize<'de> for Data<Vec<Course>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            content: Vec<Course>,
        }
        let i = I::deserialize(deserializer)?;
        Ok(Data(i.content))
    }
}

/// Course info
#[derive(Debug, Deserialize)]
pub struct Course {
    /// Course ID, for sign rule, select and drop
    pub id: u32,
    /// Course name
    #[serde(rename = "courseName")]
    pub name: String,
    /// Course location
    #[serde(rename = "coursePosition")]
    pub location: String,
    /// Course schedule
    #[serde(flatten)]
    pub schedule: Schedule,
    #[serde(deserialize_with = "deserialize_category")]
    #[serde(rename = "courseNewKind2")]
    /// Course category
    pub category: Category,
    /// Course capacity
    #[serde(flatten)]
    pub capacity: Capacity,
    /// Course campus
    #[serde(deserialize_with = "deserialize_campus")]
    #[serde(rename = "courseCampus")]
    pub campus: Campus,
    /// Whether the course is selected
    pub selected: bool,
}

/// Schedule of course's start, end, pre-selection and cancellation
#[derive(Debug, Deserialize)]
pub struct Schedule {
    /// Course start time
    #[serde(deserialize_with = "deserialize_datatime")]
    #[serde(rename = "courseStartDate")]
    pub course_start: PrimitiveDateTime,
    /// Course end time
    #[serde(deserialize_with = "deserialize_datatime")]
    #[serde(rename = "courseEndDate")]
    pub course_end: PrimitiveDateTime,
    /// Course pre-selection start time
    #[serde(deserialize_with = "deserialize_datatime")]
    #[serde(rename = "courseSelectStartDate")]
    pub select_start: PrimitiveDateTime,
    /// Course pre-selection end time
    #[serde(deserialize_with = "deserialize_datatime")]
    #[serde(rename = "courseSelectEndDate")]
    pub select_end: PrimitiveDateTime,
    /// Course cancellation end time
    #[serde(deserialize_with = "deserialize_datatime")]
    #[serde(rename = "courseCancelEndDate")]
    pub cancel_end: PrimitiveDateTime,
}

/// Course category
#[derive(Debug, Deserialize)]
pub enum Category {
    /// `美育`
    Arts,
    /// `德育`
    Ethics,
    /// `劳动教育`
    Labor,
    /// `安全健康`
    Safety,
    /// `其他`
    Other,
}

fn deserialize_category<'de, D>(deserializer: D) -> Result<Category, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    match value.get("kindName").and_then(Value::as_str) {
        Some(kind_name) => match kind_name {
            "美育" => Ok(Category::Arts),
            "德育" => Ok(Category::Ethics),
            "劳动教育" => Ok(Category::Labor),
            "安全健康" => Ok(Category::Safety),
            _ => Ok(Category::Other),
        },
        None => Err(serde::de::Error::custom("missing field `kindName`")),
    }
}

/// Course capacity
#[derive(Debug, Deserialize)]
pub struct Capacity {
    /// Maximum capacity
    #[serde(rename = "courseMaxCount")]
    pub max: u32,
    /// Current selected count
    #[serde(rename = "courseCurrentCount")]
    pub current: u32,
}

/// Course campus
#[derive(Debug, Deserialize)]
pub enum Campus {
    XueYuanLu,
    ShaHe,
    All,
    Other,
}

fn deserialize_campus<'de, D>(deserializer: D) -> Result<Campus, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    match value {
        "[1]" => Ok(Campus::XueYuanLu),
        "[2]" => Ok(Campus::ShaHe),
        // 那我问你, 你一共就俩校区, 你这 ALL 和 [1]|[2] 有**区别啊
        "ALL" | "[1]|[2]" => Ok(Campus::All),
        _ => Ok(Campus::Other),
    }
}

// ====================
// 用于 query_selected
// ====================

// 由于学校的**设计导致这个与 BoyaCourse 高度相似的结构体完全无法复用
impl<'de> Deserialize<'de> for Data<Vec<Selected>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            #[serde(rename = "courseList")]
            content: Vec<J>,
        }
        #[derive(Deserialize)]
        struct J {
            #[serde(rename = "courseInfo")]
            info: Selected,
        }
        let i = I::deserialize(deserializer)?;
        let list = i.content.into_iter().map(|x| x.info).collect();

        Ok(Data(list))
    }
}

/// Selected course info
#[derive(Debug, Deserialize)]
pub struct Selected {
    /// Course ID, for drop
    pub id: u32,
    /// Course name
    #[serde(rename = "courseName")]
    pub name: String,
    /// Course location
    #[serde(rename = "coursePosition")]
    pub location: String,
    /// Course schedule
    #[serde(flatten)]
    pub schedule: Schedule,
    #[serde(deserialize_with = "deserialize_category")]
    #[serde(rename = "courseNewKind2")]
    /// Course category
    pub category: Category,
}

// ====================
// 用于 query_statistic
// ====================

impl<'de> Deserialize<'de> for Data<Statistic> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            statistical: J,
        }

        #[derive(Deserialize)]
        struct J {
            #[serde(rename = "60|博雅课程")]
            data: Statistic,
        }

        let i = I::deserialize(deserializer)?;

        Ok(Data(i.statistical.data))
    }
}

/// Course Statistics
#[derive(Debug, Deserialize)]
pub struct Statistic {
    /// 德育
    #[serde(rename = "55|德育")]
    pub ethics: Assessment,
    /// 美育
    #[serde(rename = "56|美育")]
    pub arts: Assessment,
    /// 劳动教育
    #[serde(rename = "57|劳动教育")]
    pub labor: Assessment,
    /// 安全健康
    #[serde(rename = "58|安全健康")]
    pub safety: Assessment,
}

/// Course assessment.
/// Includes required quantity,
/// selected quantity, completed quantity,
/// incomplete quantity, and failed quantity
#[derive(Debug, Deserialize)]
pub struct Assessment {
    #[serde(rename = "assessmentCount")]
    pub require: u8,
    #[serde(rename = "selectAssessmentCount")]
    pub select: u8,
    #[serde(rename = "completeAssessmentCount")]
    pub complete: u8,
    #[serde(rename = "failAssessmentCount")]
    pub fail: u8,
    #[serde(rename = "undoneAssessmentCount")]
    pub undone: u8,
}

// ====================
// 用于 query_sign_rule
// ====================

impl<'de> Deserialize<'de> for Data<Option<SignRule>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            // 这玩意几乎啥信息没有, 主办方瞎 ** 写的, 不解析了
            // #[serde(rename = "courseDesc")]
            // pub description: String,

            // 同时用于签到签退的信息
            #[serde(rename = "courseSignConfig")]
            rule: String,
        }

        let i = I::deserialize(deserializer)?;
        if i.rule.is_empty() {
            return Ok(Data(None));
        }
        let rule = i.rule.replace("\\\"", "\"");
        match serde_json::from_str::<SignRule>(&rule) {
            Ok(r) => Ok(Data(Some(r))),
            Err(_) => Ok(Data(None)),
        }
    }
}

/// Sign rule info
#[derive(Debug, Deserialize)]
pub struct SignRule {
    #[serde(deserialize_with = "deserialize_datatime")]
    #[serde(rename = "signStartDate")]
    pub checkin_start: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_datatime")]
    #[serde(rename = "signEndDate")]
    pub checkin_end: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_datatime")]
    #[serde(rename = "signOutStartDate")]
    pub checkout_start: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_datatime")]
    #[serde(rename = "signOutEndDate")]
    pub checkout_end: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_coordinate")]
    #[serde(rename = "signPointList")]
    pub coordinate: Coordinate,
}

/// Coordinate in [SignRule]
#[derive(Debug, Deserialize)]
pub struct Coordinate {
    #[serde(rename = "lng")]
    pub longitude: f64,
    #[serde(rename = "lat")]
    pub latitude: f64,
    pub radius: i32,
}

fn deserialize_coordinate<'de, D>(deserializer: D) -> Result<Coordinate, D::Error>
where
    D: Deserializer<'de>,
{
    let mut value: Vec<Coordinate> = Deserialize::deserialize(deserializer)?;
    // 搞不懂, 但经过两次测试似乎使用的是列表的最后一个值
    if !value.is_empty() {
        return Ok(value.pop().unwrap());
    }
    Err(serde::de::Error::custom("[Boya] No Coordinate"))
}

// ====================
// 用于 sign_course
// ====================

impl<'de> Deserialize<'de> for Data<SignRes> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            #[serde(rename = "signInfo")]
            info: String,
        }

        let i = I::deserialize(deserializer)?;
        let i = i.info.replace("\\\"", "\"");
        match serde_json::from_str::<SignRes>(&i) {
            Ok(s) => Ok(Data(s)),
            Err(_) => Err(serde::de::Error::custom("[Boya] Bad SignInfo")),
        }
    }
}

/// Sign result
#[derive(Debug, Deserialize)]
pub struct SignRes {
    #[serde(rename = "signIn")]
    pub checkin: SignInfo,
    #[serde(rename = "signOut")]
    pub checkout: SignInfo,
}

/// Sign in/out info
#[derive(Debug, Deserialize)]
pub struct SignInfo {
    /// longitude
    #[serde(rename = "lng")]
    pub lon: f64,
    /// latitude
    #[serde(rename = "lat")]
    pub lat: f64,
    #[serde(rename = "inSignArea")]
    pub is_ok: bool,
}
