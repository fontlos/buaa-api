use crate::{Session, SessionError};

impl Session {
    /// # Query Statistic
    /// - Need: [`boya_login`](#method.boya_login)
    /// - Input: Token from [`boya_login`](#method.boya_login)
    pub async fn boya_query_statistic(&self, token: &str) -> Result<String, SessionError> {
        let query = "{}";
        let url = "https://bykc.buaa.edu.cn/sscv/queryStatisticByUserId";
        let res = self.boya_universal_request(query, url, token).await?;
        Ok(res)
    }
}

#[tokio::test]
async fn test_boya_query_statistic() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();

    let token = session.boya_login().await.unwrap();
    let res = session.boya_query_statistic(&token).await.unwrap();
    println!("{}", res);

    session.save();
}