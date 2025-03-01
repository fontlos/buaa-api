use super::BoyaAPI;
use super::{_BoyaStatistics, BoyaStatistic};

impl BoyaAPI {
    /// # Query Statistic
    pub async fn query_statistic(&self) -> crate::Result<BoyaStatistic> {
        let query = "{}";
        let url = "https://bykc.buaa.edu.cn/sscv/queryStatisticByUserId";
        let res = self.universal_request(query, url).await?;
        let res = serde_json::from_str::<_BoyaStatistics>(&res)?;
        Ok(res.data)
    }

    pub async fn query_statistic_vpn(&self) -> crate::Result<BoyaStatistic> {
        let query = "{}";
        let url = "https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryStatisticByUserId";
        let res = self.universal_request(query, url).await?;
        let res = serde_json::from_str::<_BoyaStatistics>(&res)?;
        Ok(res.data)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::utils::env;

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_statistic() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password).unwrap();
        context.with_cookies("cookie.json").unwrap();
        context.login().await.unwrap();

        let boya = context.boya();
        boya.login().await.unwrap();

        let res = boya.query_statistic().await.unwrap();

        println!("{:?}", res);

        context.save_cookie("cookie.json");
    }
}
