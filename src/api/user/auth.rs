use crate::api::Location;
use crate::utils;

impl super::UserApi {
    /// # User Center Login
    pub async fn login(&self) -> crate::Result<()> {
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && self.cred.load().sso.is_expired() {
            self.api::<crate::api::Core>().login().await?;
        }

        let time = utils::get_time_millis();
        // 获取 JSESSIONID
        self.get(format!(
            "https://uc.buaa.edu.cn/api/uc/status?selfTimestamp={time}"
        ))
        .send()
        .await?;
        // 验证  JSESSIONID
        // 会经历 4 次重定向
        self.get("https://uc.buaa.edu.cn/api/login?target=https://uc.buaa.edu.cn/#/user/login")
            .send()
            .await?;
        self.cred.refresh(Location::Sso);
        Ok(())
    }
}
