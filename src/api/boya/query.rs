use time::Date;

use super::{
    _BoyaCourses, _BoyaDetail, _BoyaSelecteds, _BoyaStatistics, BoyaCourse, BoyaDetail,
    BoyaSelected, BoyaStatistic,
};

impl super::BoyaAPI {
    /// # Query Course
    pub async fn query_course(&self) -> crate::Result<Vec<BoyaCourse>> {
        let query = "{\"pageNumber\":1,\"pageSize\":10}";
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryStudentSemesterCourseByPage
        let url = "https://bykc.buaa.edu.cn/sscv/queryStudentSemesterCourseByPage";
        let res = self.universal_request(query, url).await?;
        let res = serde_json::from_str::<_BoyaCourses>(&res)?;
        Ok(res.data)
    }

    pub async fn query_detail(&self, id: u32) -> crate::Result<BoyaDetail> {
        let query = format!("{{\"id\":{id}}}");
        let url = "https://bykc.buaa.edu.cn/sscv/queryCourseById";
        let res = self.universal_request(&query, url).await?;
        let res = serde_json::from_str::<_BoyaDetail>(&res)?;
        Ok(res.data)
    }

    /// # Query Selected Courses
    /// Date should like `year-month-day`
    pub async fn query_selected(&self, start: Date, end: Date) -> crate::Result<Vec<BoyaSelected>> {
        let query =
            format!("{{\"startDate\":\"{start} 00:00:00\",\"endDate\":\"{end} 00:00:00\"}}");
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryChosenCourse
        let url = "https://bykc.buaa.edu.cn/sscv/queryChosenCourse";
        let res = self.universal_request(&query, url).await?;
        let res = serde_json::from_str::<_BoyaSelecteds>(&res)?;
        Ok(res.data)
    }

    /// # Query Statistic
    pub async fn query_statistic(&self) -> crate::Result<BoyaStatistic> {
        let query = "{}";
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryStatisticByUserId
        let url = "https://bykc.buaa.edu.cn/sscv/queryStatisticByUserId";
        let res = self.universal_request(query, url).await?;
        let res = serde_json::from_str::<_BoyaStatistics>(&res)?;
        Ok(res.data)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::utils::{self, env};

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_course() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);
        // 2025.5.18 3pm 我们也成功支持 SSO 的自动刷新了
        // 现在真正可以直接调用 API 无需预处理了
        // context.login().await.unwrap();

        let boya = context.boya();
        // 2025.5.17 2pm 现在至少 Boya 的 API 是支持自动刷新的
        // boya.login().await.unwrap();

        let res = match boya.query_course().await {
            Ok(s) => s,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };
        println!("{:?}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_detail() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);

        let boya = context.boya();

        let res = boya.query_detail(7882).await.unwrap();
        println!("{:?}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_selected() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);
        context.login().await.unwrap();

        let boya = context.boya();
        boya.login().await.unwrap();

        let start = utils::parse_date("2024-08-26");
        let end = utils::parse_date("2024-12-29");

        let res = boya.query_selected(start, end).await.unwrap();
        println!("{:?}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_statistic() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);
        context.login().await.unwrap();

        let boya = context.boya();
        boya.login().await.unwrap();

        let res = boya.query_statistic().await.unwrap();

        println!("{:?}", res);
    }
}
