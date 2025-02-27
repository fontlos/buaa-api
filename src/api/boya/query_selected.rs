use serde::{Deserialize, Deserializer};
use time::Date;

use super::BoyaAPI;

use super::query_course::{BoyaKind, BoyaTime, deserialize_boya_kind};

// 由于学校的抽象设计导致这个与 BoyaCourse 高度相似的结构体完全无法复用
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

impl BoyaAPI {
    /// # Query Selected Courses
    /// Date should like `year-month-day`
    pub async fn query_selected(&self, start: Date, end: Date) -> crate::Result<Vec<BoyaSelected>> {
        let query =
            format!("{{\"startDate\":\"{start} 00:00:00\",\"endDate\":\"{end} 00:00:00\"}}");
        let url = "https://bykc.buaa.edu.cn/sscv/queryChosenCourse";
        let res = self.universal_request(&query, url).await?;
        let res = serde_json::from_str::<BoyaSelecteds>(&res)?;
        Ok(res.data)
    }

    pub async fn query_selected_vpn(
        &self,
        start: Date,
        end: Date,
    ) -> crate::Result<Vec<BoyaSelected>> {
        let query =
            format!("{{\"startDate\":\"{start} 00:00:00\",\"endDate\":\"{end} 00:00:00\"}}");
        let url = "https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryChosenCourse";
        let res = self.universal_request(&query, url).await?;
        let res = serde_json::from_str::<BoyaSelecteds>(&res)?;
        Ok(res.data)
    }
}
#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::utils::{self, env};

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_selected() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password).unwrap();
        context.with_cookies("cookie.json").unwrap();
        context.login().await.unwrap();

        let boya = context.boya();
        boya.login().await.unwrap();

        let start = utils::parse_date("2024-08-26");
        let end = utils::parse_date("2024-12-29");

        let res = boya.query_selected(start, end).await.unwrap();
        println!("{:?}", res);

        context.save_cookie("cookie.json");
    }
}
