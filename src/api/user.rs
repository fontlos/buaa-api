//! User Center API

use crate::{utils, SharedResources};

impl SharedResources {
    /// # User Center Login
    /// - Need: [`sso_login`](#method.sso_login) <br>
    pub async fn user_login(&self) -> crate::Result<()> {
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
    /// - Need: [`user_login`](#method.user_login) <br>
    /// - Output: `String`, JSON includes name and username, etc
    pub async fn user_get_state(&self) -> crate::Result<String> {
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
