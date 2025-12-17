use serde::{Deserialize, Deserializer};
use serde_json::Value;
use time::PrimitiveDateTime;

use crate::utils::deserialize_datetime;

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
    #[serde(deserialize_with = "deserialize_campuses")]
    #[serde(rename = "courseCampusList")]
    pub campuses: Vec<Campus>,
    /// Sign configuration
    #[serde(deserialize_with = "deserialize_sign")]
    #[serde(rename = "courseSignConfig")]
    pub sign_config: Option<SignConfig>,
    /// Whether the course is selected
    pub selected: bool,
}

/// Schedule of course's start, end, pre-selection and cancellation
#[derive(Debug, Deserialize)]
pub struct Schedule {
    /// Course start time
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "courseStartDate")]
    pub course_start: PrimitiveDateTime,
    /// Course end time
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "courseEndDate")]
    pub course_end: PrimitiveDateTime,
    /// Course pre-selection start time
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "courseSelectStartDate")]
    pub select_start: PrimitiveDateTime,
    /// Course pre-selection end time
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "courseSelectEndDate")]
    pub select_end: PrimitiveDateTime,
    /// Course cancellation end time
    #[serde(deserialize_with = "deserialize_datetime")]
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
    /// `学院路`
    XueYuanLu,
    /// `沙河`
    ShaHe,
    /// 杭州校区
    HangZhou,
    /// `未知校区`
    Unknown,
    /// `全部校区`
    All,
}

fn deserialize_campuses<'de, D>(deserializer: D) -> Result<Vec<Campus>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Vec<&str> = Deserialize::deserialize(deserializer)?;
    let mut campuses = Vec::with_capacity(value.len());
    // 最 ** 的设计, 你一共几个校区啊, 就非要三个都显示或只显示一个全部校区并存呗
    for c in value {
        match c {
            "全部校区" => campuses.push(Campus::All),
            "学院路校区" => campuses.push(Campus::XueYuanLu),
            "沙河校区" => campuses.push(Campus::ShaHe),
            "杭州校区" => campuses.push(Campus::HangZhou),
            _ => campuses.push(Campus::Unknown),
        }
    }
    Ok(campuses)
}

fn deserialize_sign<'de, D>(deserializer: D) -> Result<Option<SignConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: String = String::deserialize(deserializer)?;
    if value.is_empty() {
        Ok(None)
    } else {
        let value = value.replace("\\\"", "\"");
        serde_json::from_str::<SignConfig>(&value)
            .map(Some)
            .map_err(|_| serde::de::Error::custom("Bad CourseSignConfig"))
    }
}

impl<'de> Deserialize<'de> for Data<Option<SignConfig>> {
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
            Ok(Data(None))
        } else {
            let rule = i.rule.replace("\\\"", "\"");
            serde_json::from_str::<SignConfig>(&rule)
                .map(|r| Data(Some(r)))
                .map_err(|_| serde::de::Error::custom("Bad CourseSignConfig"))
        }
    }
}

/// Sign rule info
#[derive(Debug, Deserialize)]
pub struct SignConfig {
    /// Check in start time
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "signStartDate")]
    pub checkin_start: PrimitiveDateTime,
    /// Check in end time
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "signEndDate")]
    pub checkin_end: PrimitiveDateTime,
    /// Check out start time
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "signOutStartDate")]
    pub checkout_start: PrimitiveDateTime,
    /// Check out end time
    #[serde(deserialize_with = "deserialize_datetime")]
    #[serde(rename = "signOutEndDate")]
    pub checkout_end: PrimitiveDateTime,
    /// Coordinate for check in/out
    #[serde(deserialize_with = "deserialize_coordinate")]
    #[serde(rename = "signPointList")]
    pub coordinate: Coordinate,
}

/// Coordinate in [SignRule]
#[derive(Debug, Deserialize)]
pub struct Coordinate {
    /// Longitude
    #[serde(rename = "lng")]
    pub longitude: f64,
    /// Latitude
    #[serde(rename = "lat")]
    pub latitude: f64,
    /// Radius
    pub radius: f64,
}

fn deserialize_coordinate<'de, D>(deserializer: D) -> Result<Coordinate, D::Error>
where
    D: Deserializer<'de>,
{
    let mut value: Vec<Coordinate> = Deserialize::deserialize(deserializer)?;
    // 搞不懂, 但经过两次测试似乎使用的是列表的最后一个值
    value
        .pop()
        .ok_or_else(|| serde::de::Error::custom("No Coordinate"))
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
/// failed quantity, and undone quantity
#[derive(Debug, Deserialize)]
pub struct Assessment {
    /// Required quantity
    #[serde(rename = "assessmentCount")]
    pub require: u8,
    /// Selected quantity
    #[serde(rename = "selectAssessmentCount")]
    pub select: u8,
    /// Completed quantity
    #[serde(rename = "completeAssessmentCount")]
    pub complete: u8,
    /// Failed quantity
    #[serde(rename = "failAssessmentCount")]
    pub fail: u8,
    /// Undone quantity
    #[serde(rename = "undoneAssessmentCount")]
    pub undone: u8,
}

// ====================
// 用于 query_sign_rule
// ====================

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
            Err(_) => Err(serde::de::Error::custom("Bad SignInfo")),
        }
    }
}

/// Sign result
#[derive(Debug, Deserialize)]
pub struct SignRes {
    /// Check in info
    #[serde(rename = "signIn")]
    pub checkin: SignInfo,
    /// Check out info
    #[serde(rename = "signOut")]
    pub checkout: SignInfo,
}

/// Sign in/out info
#[derive(Debug, Deserialize)]
pub struct SignInfo {
    /// Longitude
    #[serde(rename = "lng")]
    pub lon: f64,
    /// Latitude
    #[serde(rename = "lat")]
    pub lat: f64,
    /// Whether the sign in/out is successful
    #[serde(rename = "inSignArea")]
    pub is_ok: bool,
}
