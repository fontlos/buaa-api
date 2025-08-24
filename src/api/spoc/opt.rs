use super::{SpocSchedule, SpocWeek};

impl super::SpocApi {
    /// Get current week
    pub async fn get_week(&self) -> crate::Result<SpocWeek> {
        // SQL ID 似乎可以是固定值, 应该是用于鉴权的, 不知道是否会过期
        let query = serde_json::json!({
            "sqlid": "17275975753144ed8d6fe15425677f752c936d97de1bab76"
        });
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryOne";
        let res: SpocWeek = self.universal_request(url, &query).await?;
        Ok(res)
    }

    pub async fn get_week_schedule(&self, week: &SpocWeek) -> crate::Result<Vec<SpocSchedule>> {
        // 后面三个值分别是开始日期, 结束日期和学年学期
        let query = serde_json::json!({
            "sqlid": "17138556333937a86d7c38783bc62811e7c6bb5ef955a",
            "zksrq": week.date.0,
            "zjsrq": week.date.1,
            "xnxq": week.term
        });
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryList";
        let res: Vec<SpocSchedule> = self.universal_request(url, &query).await?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[ignore]
    #[tokio::test]
    async fn test_spoc_get_schedule() {
        let context = Context::with_auth("./data");

        let spoc = context.spoc();

        let res = spoc.get_week().await.unwrap();
        println!("{:?}", res);
        let res = spoc.get_week_schedule(&res).await.unwrap();
        println!("{:?}", res);
        // context.save_auth("./data");
    }
}
