use super::{Schedule, Week};

impl super::SpocApi {
    /// Get current week
    pub async fn get_week(&self) -> crate::Result<Week> {
        // SQL ID 似乎可以是固定值, 应该是用于鉴权的, 不知道是否会过期
        let query = serde_json::json!({
            "sqlid": "17275975753144ed8d6fe15425677f752c936d97de1bab76"
        });
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryOne";
        let res: Week = self.universal_request(url, &query).await?;
        Ok(res)
    }

    /// Get schedule of a week
    pub async fn get_week_schedule(&self, week: &Week) -> crate::Result<Vec<Schedule>> {
        // 后面三个值分别是开始日期, 结束日期和学年学期
        let query = serde_json::json!({
            "sqlid": "17138556333937a86d7c38783bc62811e7c6bb5ef955a",
            "zksrq": week.date.0,
            "zjsrq": week.date.1,
            "xnxq": week.term
        });
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryList";
        let res: Vec<Schedule> = self.universal_request(url, &query).await?;
        Ok(res)
    }
}
