use serde::{Deserialize, Deserializer};

use std::ops::Deref;

use super::BoyaAPI;

use super::query_course::{deserialize_boya_kind, BoyaKind, BoyaTime};

#[cfg(feature = "table")]
use super::query_course::{
    tabled_boya_kind, tabled_boya_name, tabled_boya_position, tabled_boya_time,
};

#[derive(Debug, Deserialize)]
pub struct BoyaSelecteds {
    #[serde(deserialize_with = "deserialize_boya_selecteds")]
    data: Vec<BoyaSelected>,
}

// 自动解引用, 多数情况下无需访问 data 字段
impl Deref for BoyaSelecteds {
    type Target = Vec<BoyaSelected>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// 开启 table feature 时实现 Display
#[cfg(feature = "table")]
impl std::fmt::Display for BoyaSelecteds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let table = crate::utils::table(self);
        writeln!(f, "{}", table)
    }
}

// 如果真的需要访问 data 字段, 可以使用 data 方法
impl BoyaSelecteds {
    pub fn data(self) -> Vec<BoyaSelected> {
        self.data
    }
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

#[cfg_attr(feature = "table", derive(tabled::Tabled))]
#[derive(Debug, Deserialize)]
pub struct BoyaSelected {
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
}

impl BoyaAPI {
    /// # Query Selected Courses
    pub async fn query_selected(&self) -> crate::Result<BoyaSelecteds> {
        let query = "{\"startDate\":\"2024-08-26 00:00:00\",\"endDate\":\"2024-12-29 00:00:00\"}";
        let url = "https://bykc.buaa.edu.cn/sscv/queryChosenCourse";
        let res = self.universal_request(query, url).await?;
        let res = serde_json::from_str::<BoyaSelecteds>(&res)?;
        Ok(res)
    }
}

#[tokio::test]
async fn test_boya_query_selected() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let context = crate::Context::new();
    context.with_cookies("cookie.json");
    context.login(&username, &password).await.unwrap();

    let boya = context.boya();
    boya.login().await.unwrap();

    let res = boya.query_selected().await.unwrap();
    println!("{:?}", res);

    context.save();
}
