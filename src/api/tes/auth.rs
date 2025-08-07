use crate::error::{Error, Location};

impl super::TesApi {
    /// Teacher Evaluation System Login
    pub async fn login(&self) -> crate::Result<()> {
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && self.cred.load().sso.is_expired() {
            self.api::<crate::api::Core>().login().await?;
        }

        // 登录
        let login_url =
            "https://sso.buaa.edu.cn/login?service=https%3A%2F%2Fspoc.buaa.edu.cn%2Fpjxt%2Fcas";
        let res = self.get(login_url).send().await?;
        if res.url().as_str() == login_url {
            return Err(Error::auth_expired(Location::Sso));
        }
        self.cred.update(|c| {
            // 刷新 SSO 时效
            c.sso.refresh(5400);
        });
        Ok(())
    }
}
