use time::Date;

use super::BoyaAPI;
use super::{_BoyaSelecteds, BoyaSelected};

impl BoyaAPI {
    /// # Query Selected Courses
    /// Date should like `year-month-day`
    pub async fn query_selected(&self, start: Date, end: Date) -> crate::Result<Vec<BoyaSelected>> {
        let query =
            format!("{{\"startDate\":\"{start} 00:00:00\",\"endDate\":\"{end} 00:00:00\"}}");
        let url = "https://bykc.buaa.edu.cn/sscv/queryChosenCourse";
        let res = self.universal_request(&query, url).await?;
        let res = serde_json::from_str::<_BoyaSelecteds>(&res)?;
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
        let res = serde_json::from_str::<_BoyaSelecteds>(&res)?;
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
