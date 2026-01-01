use reqwest::Method;

use super::{Course, Payload, Schedule, Week, Res};

impl super::SpocApi {
    /// Get current week
    pub async fn get_week(&self) -> crate::Result<Week> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryOne";
        // SQL ID 是固定值, 应该是对应的数据库键什么的
        let json = serde_json::json!({
            "sqlid": "17275975753144ed8d6fe15425677f752c936d97de1bab76"
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(url, Method::POST, payload).await?;
        let res: Week = Res::parse(&bytes)?;
        Ok(res)
    }

    /// Get schedule of a week
    pub async fn get_week_schedule(&self, week: &Week) -> crate::Result<Vec<Schedule>> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryList";
        // 后面三个值分别是开始日期, 结束日期和学年学期
        let json = serde_json::json!({
            "sqlid": "17138556333937a86d7c38783bc62811e7c6bb5ef955a",
            "zksrq": week.date.0,
            "zjsrq": week.date.1,
            "xnxq": week.term
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(url, Method::POST, payload).await?;
        let res: Vec<Schedule> = Res::parse(&bytes)?;
        Ok(res)
    }

    /// # Query courses
    ///
    /// `term` format: "yyyy-yyyyt", e.g. "2025-20261" for 2025 fall semester.
    /// Can get from [Week::term]
    pub async fn query_courses(&self, term: &str) -> crate::Result<Vec<Course>> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/jxkj/queryKclb";
        let query = [("xnxq", term)];
        let payload = Payload::Query(&query);
        let bytes = self.universal_request(url, Method::GET, payload).await?;
        let res: Vec<Course> = Res::parse(&bytes)?;
        Ok(res)
    }
}
