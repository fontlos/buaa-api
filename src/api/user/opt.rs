use crate::utils;

impl super::UserCenterAPI {
    /// # User Center Login
    pub async fn login(&self) -> crate::Result<()> {
        let time = utils::get_time();
        // 获取 JSESSIONID
        self.get(format!(
            "https://uc.buaa.edu.cn/api/uc/status?selfTimestamp={}",
            time
        ))
        .send()
        .await?;
        // 验证  JSESSIONID
        // 会经历 4 次重定向
        self.get("https://uc.buaa.edu.cn/api/login?target=https://uc.buaa.edu.cn/#/user/login")
            .send()
            .await?;
        Ok(())
    }

    /// # Get User Center state
    /// - Output: `String`, JSON includes name and username, etc
    pub async fn get_state(&self) -> crate::Result<String> {
        let time = utils::get_time();
        // 获取登录状态
        let res = self
            .get(format!(
                "https://uc.buaa.edu.cn/api/uc/status?selfTimestamp={}",
                time
            ))
            .send()
            .await?;
        let state = res.text().await?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::utils::env;

    #[ignore]
    #[tokio::test]
    async fn test_user() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password).unwrap();
        context.with_cookies("cookie.json").unwrap();
        context.login().await.unwrap();

        let user = context.user();
        user.login().await.unwrap();

        let state = user.get_state().await.unwrap();
        println!("{}", state);

        context.save_cookie("cookie.json");
    }
}
