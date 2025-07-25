use crate::utils;

impl super::UserAPI {
    /// # Get User Center state
    /// - Output: `String`, JSON includes name and username, etc
    pub async fn get_state(&self) -> crate::Result<String> {
        let time = utils::get_time_millis();
        // 获取登录状态
        let res = self
            .get(format!(
                "https://uc.buaa.edu.cn/api/uc/status?selfTimestamp={time}"
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

    #[ignore]
    #[tokio::test]
    async fn test_user() {
        let context = Context::with_auth("./data");

        let user = context.user();
        user.login().await.unwrap();

        let state = user.get_state().await.unwrap();
        println!("{}", state);
    }
}
