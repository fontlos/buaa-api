use crate::utils::time::DateTime;

impl super::UserApi {
    /// # Get User Center state
    /// - Output: `String`, JSON includes name and username, etc
    pub async fn get_state(&self) -> crate::Result<String> {
        let time = DateTime::millis();
        // 获取登录状态
        let res = self
            .client
            .get(format!(
                "https://uc.buaa.edu.cn/api/uc/status?selfTimestamp={time}"
            ))
            .send()
            .await?;
        let state = res.text().await?;
        Ok(state)
    }
}
