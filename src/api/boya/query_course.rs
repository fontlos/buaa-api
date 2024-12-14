use serde::{Deserialize, Deserializer};
use serde_json::Value;
use time::{format_description, PrimitiveDateTime};

use std::ops::Deref;

use super::BoyaAPI;

#[derive(Debug, Deserialize)]
pub struct BoyaCourses {
    #[serde(deserialize_with = "deserialize_boya_courses")]
    data: Vec<BoyaCourse>,
}

// 自动解引用, 多数情况下无需访问 data 字段
impl Deref for BoyaCourses {
    type Target = Vec<BoyaCourse>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// 开启 table feature 时实现 Display
#[cfg(feature = "table")]
impl std::fmt::Display for BoyaCourses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let table = crate::utils::table(self);
        writeln!(f, "{}", table)
    }
}

// 如果真的需要访问 data 字段, 可以使用 data 方法
impl BoyaCourses {
    pub fn data(self) -> Vec<BoyaCourse> {
        self.data
    }
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

#[cfg_attr(feature = "table", derive(tabled::Tabled))]
#[derive(Debug, Deserialize)]
pub struct BoyaCourse {
    // 课程 ID
    pub id: u32,
    // 课程名
    #[serde(rename = "courseName")]
    #[cfg_attr(feature = "table", tabled(display_with = "tabled_boya_name"))]
    pub name: String,
    // 地点
    #[serde(rename = "coursePosition")]
    #[cfg_attr(feature = "table", tabled(display_with = "tabled_boya_position"))]
    pub position: String,
    // 开始结束和预选时间
    #[serde(flatten)]
    #[cfg_attr(feature = "table", tabled(display_with = "tabled_boya_time"))]
    pub time: BoyaTime,
    #[serde(deserialize_with = "deserialize_boya_kind")]
    #[serde(rename = "courseNewKind2")]
    #[cfg_attr(feature = "table", tabled(display_with = "tabled_boya_kind"))]
    // 课程种类
    pub kind: BoyaKind,
    #[serde(flatten)]
    #[cfg_attr(feature = "table", tabled(display_with = "tabled_boya_capacity"))]
    pub capacity: BoyaCapacity,
    // 开设校区
    #[serde(deserialize_with = "deserialize_boya_campus")]
    #[serde(rename = "courseCampus")]
    #[cfg_attr(feature = "table", tabled(display_with = "tabled_boya_campus"))]
    pub campus: BoyaCampus,
    // 是否已选
    pub selected: bool,
}

#[cfg(feature = "table")]
pub(crate) fn tabled_boya_name(s: &str) -> String {
    textwrap::wrap(s, 18).join("\n")
}

#[cfg(feature = "table")]
pub(crate) fn tabled_boya_position(s: &str) -> String {
    textwrap::wrap(s, 15).join("\n")
}

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

// 和 SmartClass 共用的
pub(crate) fn deserialize_time<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let format_string =
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

    let s: String = Deserialize::deserialize(deserializer)?;

    PrimitiveDateTime::parse(&s, &format_string).map_err(|e| serde::de::Error::custom(e))
}

#[cfg(feature = "table")]
pub(crate) fn tabled_boya_time(time: &BoyaTime) -> String {
    let format_string = format_description::parse("[year].[month].[day] [hour]:[minute]").unwrap();

    let formatted_course_start = time.course_start.format(&format_string).unwrap();
    let formatted_course_end = time.course_end.format(&format_string).unwrap();
    let formatted_select_start = time.select_start.format(&format_string).unwrap();
    let formatted_select_end = time.select_end.format(&format_string).unwrap();

    format!(
        "             CourseTime\n{} - {}\n             SelectTime\n{} - {}",
        formatted_course_start, formatted_course_end, formatted_select_start, formatted_select_end
    )
}

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

pub(crate) fn deserialize_boya_kind<'de, D>(deserializer: D) -> Result<BoyaKind, D::Error>
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

#[cfg(feature = "table")]
pub(crate) fn tabled_boya_kind(capacity: &BoyaKind) -> String {
    match capacity {
        BoyaKind::Arts => "美育".to_string(),
        BoyaKind::Ethics => "德育".to_string(),
        BoyaKind::Labor => "劳动教育".to_string(),
        BoyaKind::Safety => "安全健康".to_string(),
        BoyaKind::Other => "其他".to_string(),
    }
}

#[derive(Debug, Deserialize)]
pub struct BoyaCapacity {
    #[serde(rename = "courseMaxCount")]
    pub max: u32,
    #[serde(rename = "courseCurrentCount")]
    pub current: u32,
}

#[cfg(feature = "table")]
fn tabled_boya_capacity(capacity: &BoyaCapacity) -> String {
    format!("{} / {}", capacity.current, capacity.max)
}

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

#[cfg(feature = "table")]
fn tabled_boya_campus(capacity: &BoyaCampus) -> String {
    match capacity {
        BoyaCampus::XueYuanLu => "学院路".to_string(),
        BoyaCampus::ShaHe => "沙河".to_string(),
        BoyaCampus::All => "全部".to_string(),
        BoyaCampus::Other => "其他".to_string(),
    }
}

impl BoyaAPI {
    /// # Query Course
    /// - Need: [`boya_login`](#method.boya_login)
    pub async fn query_course(&self) -> crate::Result<BoyaCourses> {
        let query = "{\"pageNumber\":1,\"pageSize\":10}";
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryStudentSemesterCourseByPage
        let url = "https://bykc.buaa.edu.cn/sscv/queryStudentSemesterCourseByPage";
        let res = self.universal_request(query, url).await?;
        let res = serde_json::from_str::<BoyaCourses>(&res)?;
        Ok(res)
    }
}

#[tokio::test]
async fn test_boya_query_course() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let context = crate::Context::new();
    context.with_cookies("cookie.json");

    context.login(&username, &password).await.unwrap();

    let boya = context.boya();
    boya.login().await.unwrap();
    let res = match boya.query_course().await {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    println!("{:?}", res);

    context.save();
}
