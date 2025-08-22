use time::Date;

use super::{
    _BoyaCourses, _BoyaDetail, _BoyaSelecteds, _BoyaStatistics, BoyaCourse, BoyaSelected,
    BoyaSignRule, BoyaStatistic,
};

impl super::BoyaApi {
    /// # Query Course
    pub async fn query_course(&self) -> crate::Result<Vec<BoyaCourse>> {
        let query = serde_json::json!({
            "pageNumber": 1,
            "pageSize": 10,
        });
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryStudentSemesterCourseByPage
        let url = "https://bykc.buaa.edu.cn/sscv/queryStudentSemesterCourseByPage";
        let res = self.universal_request(url, &query).await?;
        let res = serde_json::from_str::<_BoyaCourses>(&res)?;
        Ok(res.data)
    }

    // 查询签到规则, 包括签到签退的时间地点
    pub async fn query_sign_rule(&self, id: u32) -> crate::Result<Option<BoyaSignRule>> {
        let query = serde_json::json!({
            "id": id,
        });
        let url = "https://bykc.buaa.edu.cn/sscv/queryCourseById";
        let res = self.universal_request(url, &query).await?;
        let res = serde_json::from_str::<_BoyaDetail>(&res)?;
        Ok(res.data)
    }

    /// # Query Selected Courses
    /// Date should like `year-month-day`
    pub async fn query_selected(&self, start: Date, end: Date) -> crate::Result<Vec<BoyaSelected>> {
        // TODO: 查询其他时间范围也没用, 找个接口查询本学期时间范围, 封装进去无需参数算了
        let query = serde_json::json!({
            "startDate": format!("{} 00:00:00", start),
            "endDate": format!("{} 00:00:00", end),
        });
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryChosenCourse
        let url = "https://bykc.buaa.edu.cn/sscv/queryChosenCourse";
        let res = self.universal_request(url, &query).await?;
        let res = serde_json::from_str::<_BoyaSelecteds>(&res)?;
        Ok(res.data)
    }

    /// # Query Statistic
    pub async fn query_statistic(&self) -> crate::Result<BoyaStatistic> {
        let query = serde_json::json!({});
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryStatisticByUserId
        let url = "https://bykc.buaa.edu.cn/sscv/queryStatisticByUserId";
        let res = self.universal_request(url, &query).await?;
        let res = serde_json::from_str::<_BoyaStatistics>(&res)?;
        Ok(res.data)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::utils;

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_course() {
        let context = Context::with_auth("./data");
        // 2025.5.18 15:00 我们也成功支持 SSO 的自动刷新了
        // 现在真正可以直接调用 API 无需预处理了
        // context.login().await.unwrap();

        let boya = context.boya();
        // 2025.5.17 14:00 现在至少 Boya 的 API 是支持自动刷新的
        // boya.login().await.unwrap();

        let res = boya.query_course().await.unwrap();
        println!("{:?}", res);

        context.save_auth("./data");
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_detail() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let res = boya.query_sign_rule(7882).await.unwrap();
        println!("{:?}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_selected() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let start = utils::parse_date("2024-08-26");
        let end = utils::parse_date("2024-12-29");

        let res = boya.query_selected(start, end).await.unwrap();
        println!("{:?}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_statistic() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let res = boya.query_statistic().await.unwrap();

        println!("{:?}", res);
    }
}
