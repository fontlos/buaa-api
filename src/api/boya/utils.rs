use serde::{Deserialize, Deserializer};
use serde_json::Value;
use time::PrimitiveDateTime;

use crate::utils::deserialize_time;

// ====================
// 用于 query_course
// ====================

#[derive(Deserialize)]
pub(super) struct _BoyaCourses {
    #[serde(deserialize_with = "deserialize_boya_courses")]
    pub data: Vec<BoyaCourse>,
}

// 用于直接解析到 BoyaCourses 的函数
fn deserialize_boya_courses<'de, D>(deserializer: D) -> Result<Vec<BoyaCourse>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Intermediate {
        content: Vec<BoyaCourse>,
    }

    let intermediate = Intermediate::deserialize(deserializer)?;
    Ok(intermediate.content)
}

#[derive(Debug, Deserialize)]
pub struct BoyaCourse {
    // 课程 ID
    pub id: u32,
    // 课程名
    #[serde(rename = "courseName")]
    pub name: String,
    // 地点
    #[serde(rename = "coursePosition")]
    pub position: String,
    // 开始结束和预选时间
    #[serde(flatten)]
    pub time: BoyaTime,
    #[serde(deserialize_with = "deserialize_boya_kind")]
    #[serde(rename = "courseNewKind2")]
    // 课程种类
    pub kind: BoyaKind,
    #[serde(flatten)]
    pub capacity: BoyaCapacity,
    // 开设校区
    #[serde(deserialize_with = "deserialize_boya_campus")]
    #[serde(rename = "courseCampus")]
    pub campus: BoyaCampus,
    // 是否已选
    pub selected: bool,
}

/// Boya course's start, end, pre-selection and cancellation times
#[derive(Debug, Deserialize)]
pub struct BoyaTime {
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(rename = "courseStartDate")]
    pub course_start: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(rename = "courseEndDate")]
    pub course_end: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(rename = "courseSelectStartDate")]
    pub select_start: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(rename = "courseSelectEndDate")]
    pub select_end: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(rename = "courseCancelEndDate")]
    pub cancel_end: PrimitiveDateTime,
}

/// Boya course's kind
#[derive(Debug, Deserialize)]
pub enum BoyaKind {
    /// 美育
    Arts,
    /// 德育
    Ethics,
    /// 劳动教育
    Labor,
    /// 安全健康
    Safety,
    /// 其他
    Other,
}

fn deserialize_boya_kind<'de, D>(deserializer: D) -> Result<BoyaKind, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    match value.get("kindName").and_then(Value::as_str) {
        Some(kind_name) => match kind_name {
            "美育" => Ok(BoyaKind::Arts),
            "德育" => Ok(BoyaKind::Ethics),
            "劳动教育" => Ok(BoyaKind::Labor),
            "安全健康" => Ok(BoyaKind::Safety),
            _ => Ok(BoyaKind::Other),
        },
        None => Err(serde::de::Error::custom("missing field `kindName`")),
    }
}

/// Boya course's capacity
#[derive(Debug, Deserialize)]
pub struct BoyaCapacity {
    #[serde(rename = "courseMaxCount")]
    pub max: u32,
    #[serde(rename = "courseCurrentCount")]
    pub current: u32,
}

/// Boya course's campus
#[derive(Debug, Deserialize)]
pub enum BoyaCampus {
    XueYuanLu,
    ShaHe,
    All,
    Other,
}

fn deserialize_boya_campus<'de, D>(deserializer: D) -> Result<BoyaCampus, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    match value {
        "[1]" => Ok(BoyaCampus::XueYuanLu),
        "[2]" => Ok(BoyaCampus::ShaHe),
        "ALL" => Ok(BoyaCampus::All),
        _ => Ok(BoyaCampus::Other),
    }
}

// ====================
// 用于 query_selected
// ====================

// 由于学校的抽象设计导致这个与 BoyaCourse 高度相似的结构体完全无法复用
#[derive(Deserialize)]
pub(super) struct _BoyaSelecteds {
    #[serde(deserialize_with = "deserialize_boya_selecteds")]
    pub data: Vec<BoyaSelected>,
}

fn deserialize_boya_selecteds<'de, D>(deserializer: D) -> Result<Vec<BoyaSelected>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Intermediate {
        #[serde(rename = "courseList")]
        content: Vec<IntermediateBoyaSelected>,
    }
    #[derive(Deserialize)]
    struct IntermediateBoyaSelected {
        #[serde(rename = "courseInfo")]
        info: BoyaSelected,
    }
    let intermediate = Intermediate::deserialize(deserializer)?;
    let course_list = intermediate.content;

    Ok(course_list.into_iter().map(|x| x.info).collect())
}

/// Selected Boya courses
#[derive(Debug, Deserialize)]
pub struct BoyaSelected {
    // 课程 ID
    pub id: u32,
    // 课程名
    #[serde(rename = "courseName")]
    pub name: String,
    // 地点
    #[serde(rename = "coursePosition")]
    pub position: String,
    // 开始结束和预选时间
    #[serde(flatten)]
    pub time: BoyaTime,
    #[serde(deserialize_with = "deserialize_boya_kind")]
    #[serde(rename = "courseNewKind2")]
    // 课程种类
    pub kind: BoyaKind,
}

// ====================
// 用于 query_statistic
// ====================

#[derive(Deserialize)]
pub(super) struct _BoyaStatistics {
    #[serde(deserialize_with = "deserialize_boya_statistics")]
    pub data: BoyaStatistic,
}

fn deserialize_boya_statistics<'de, D>(deserializer: D) -> Result<BoyaStatistic, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Intermediate {
        statistical: IntermediateBoyaStatisticData,
    }

    #[derive(Deserialize)]
    struct IntermediateBoyaStatisticData {
        #[serde(rename = "60|博雅课程")]
        data: BoyaStatistic,
    }

    let intermediate = Intermediate::deserialize(deserializer)?;

    Ok(intermediate.statistical.data)
}

/// Boya course's Statistics
#[derive(Debug, Deserialize)]
pub struct BoyaStatistic {
    /// 德育
    #[serde(rename = "55|德育")]
    pub ethics: BoyaAssessment,
    /// 美育
    #[serde(rename = "56|美育")]
    pub arts: BoyaAssessment,
    /// 劳动教育
    #[serde(rename = "57|劳动教育")]
    pub labor: BoyaAssessment,
    /// 安全健康
    #[serde(rename = "58|安全健康")]
    pub safety: BoyaAssessment,
}

/// Boya course's assessment. Includes required quantity, selected quantity, completed quantity, incomplete quantity, and failed quantity
#[derive(Debug, Deserialize)]
pub struct BoyaAssessment {
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
