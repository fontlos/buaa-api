use crate::api::Location;

impl super::AppApi {
    pub async fn login(&self) -> crate::Result<()> {
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && self.cred.load().sso.is_expired() {
            self.api::<crate::api::Core>().login().await?;
        }

        self.get("https://app.buaa.edu.cn/uc/wap/login")
            .send()
            .await?;

        self.cred.update(|c| {
            c.refresh(Location::Sso);
        });

        Ok(())
    }
}
