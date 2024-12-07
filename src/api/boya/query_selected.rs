use serde::{Deserialize, Deserializer};

use crate::{Session, SessionError};

use super::query_course::{deserialize_boya_kind, BoyaKind, BoyaTime};

#[cfg(feature = "table")]
use super::query_course::{
    tabled_boya_kind, tabled_boya_name, tabled_boya_position, tabled_boya_time,
};

#[derive(Debug, Deserialize)]
struct BoyaSelecteds {
    #[serde(deserialize_with = "deserialize_boya_selecteds")]
    data: Vec<BoyaSelected>,
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

impl Session {
    /// # Query Selected Course
    /// - Need: [`boya_login`](#method.boya_login)
    /// - Input: Token from [`boya_login`](#method.boya_login)
    pub async fn boya_query_selected(
        &self,
        token: &str,
    ) -> Result<Vec<BoyaSelected>, SessionError> {
        let query = "{\"startDate\":\"2024-08-26 00:00:00\",\"endDate\":\"2024-12-29 00:00:00\"}";
        let url = "https://bykc.buaa.edu.cn/sscv/queryChosenCourse";
        let res = self.boya_universal_request(query, url, token).await?;
        let res = serde_json::from_str::<BoyaSelecteds>(&res)?;
        Ok(res.data)
    }
}

#[tokio::test]
async fn test_boya_query_selected() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();

    let token = session.boya_login().await.unwrap();
    let res = session.boya_query_selected(&token).await.unwrap();
    println!("{:?}", res);

    session.save();
}
