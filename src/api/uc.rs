use crate::{Session, SessionError, utils};

impl Session{
    /// UC Login </br>
    /// This is the page you will be redirected to when you login to SSO
    pub async fn uc_login(&self) -> Result<(), SessionError> {
        let time = utils::get_time();
        // 获取 JSESSIONID
        self.get(format!("https://uc.buaa.edu.cn/api/uc/status?selfTimestamp={}", time))
            .send()
            .await?;
        // 验证  JSESSIONID
        // 会经历 4 次重定向
        self.get("https://uc.buaa.edu.cn/api/login?target=https://uc.buaa.edu.cn/#/user/login")
            .send()
            .await?;
        Ok(())
    }

    /// Get UC state, return a json string, includes name and username, etc
    pub async fn uc_get_state(&self) -> Result<String, SessionError> {
        let time = utils::get_time();
        // 获取登录状态
        let res = self.get(format!("https://uc.buaa.edu.cn/api/uc/status?selfTimestamp={}", time))
            .send()
            .await?;
        let state = res.text().await?;
        Ok(state)
    }
}
