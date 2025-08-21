use super::{_SpocRes, SpocSchedule, SpocWeek};

impl super::SpocApi {
    /// Get current week
    pub async fn get_week(&self) -> crate::Result<SpocWeek> {
        // SQL ID 似乎可以是固定值, 应该是用于鉴权的, 不知道是否会过期
        let query = r#"{"sqlid":"17275975753144ed8d6fe15425677f752c936d97de1bab76"}"#;
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryOne";
        let res = self.universal_request(url, query).await?;
        let res = serde_json::from_str::<_SpocRes<SpocWeek>>(&res)?;
        Ok(res.content)
    }

    pub async fn get_week_schedule(&self, week: &SpocWeek) -> crate::Result<Vec<SpocSchedule>> {
        // 后面三个值分别是开始日期, 结束日期和学年学期
        let query = format!(
            "{{\"sqlid\":\"17138556333937a86d7c38783bc62811e7c6bb5ef955a\",\"zksrq\":\"{}\",\"zjsrq\":\"{}\",\"xnxq\":\"{}\"}}",
            week.date.0, week.date.1, week.term
        );
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryList";
        let res = self.universal_request(url, &query).await?;
        let res = serde_json::from_str::<_SpocRes<Vec<SpocSchedule>>>(&res)?;
        Ok(res.content)
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
        // spoc.login().await.unwrap();

        let res = spoc.get_week().await.unwrap();
        println!("{:?}", res);
        // let res = spoc.get_week_schedule(&res).await.unwrap();
        // println!("{:?}", res);
        // context.save_auth("./data");
    }
}
