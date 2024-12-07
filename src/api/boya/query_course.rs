use serde::{Deserialize, Deserializer};
use serde_json::Value;
use time::{format_description, PrimitiveDateTime};

use crate::{Session, SessionError};

#[derive(Deserialize)]
struct BoyaCourses {
    #[serde(deserialize_with = "deserialize_boya_courses")]
    data: Vec<BoyaCourse>,
}

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
    AnQuan,
    DeYu,
    LaoDong,
    MeiYu,
    Other,
}

pub(crate) fn deserialize_boya_kind<'de, D>(deserializer: D) -> Result<BoyaKind, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    match value.get("kindName").and_then(Value::as_str) {
        Some(kind_name) => match kind_name {
            "安全健康" => Ok(BoyaKind::AnQuan),
            "德育" => Ok(BoyaKind::DeYu),
            "劳动教育" => Ok(BoyaKind::LaoDong),
            "美育" => Ok(BoyaKind::MeiYu),
            _ => Ok(BoyaKind::Other),
        },
        None => Err(serde::de::Error::custom("missing field `kindName`")),
    }
}

#[cfg(feature = "table")]
pub(crate) fn tabled_boya_kind(capacity: &BoyaKind) -> String {
    match capacity {
        BoyaKind::AnQuan => "安全健康".to_string(),
        BoyaKind::DeYu => "德育".to_string(),
        BoyaKind::LaoDong => "劳动教育".to_string(),
        BoyaKind::MeiYu => "美育".to_string(),
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

impl Session {
    /// # Query Course
    /// - Need: [`boya_login`](#method.boya_login)
    /// - Input: Token from [`boya_login`](#method.boya_login)
    pub async fn boya_query_course(&self, token: &str) -> Result<Vec<BoyaCourse>, SessionError> {
        let query = "{\"pageNumber\":1,\"pageSize\":10}";
        let url = "https://bykc.buaa.edu.cn/sscv/queryStudentSemesterCourseByPage";
        let res = self.boya_universal_request(query, url, token).await?;
        let res = serde_json::from_str::<BoyaCourses>(&res)?;
        Ok(res.data)
    }
}

#[tokio::test]
async fn test_boya_query_course() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();
    let token = session.boya_login().await.unwrap();
    let res = match session.boya_query_course(&token).await {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    println!("{:?}", res);
    // println!("{}", utils::table(res));

    session.save();
}
