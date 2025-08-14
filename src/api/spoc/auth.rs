use crate::api::Location;
use crate::error::Error;

impl super::SpocApi {
    /// # Spoc Login
    pub async fn login(&self) -> crate::Result<()> {
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && self.cred.load().sso.is_expired() {
            self.api::<crate::api::Core>().login().await?;
        }

        let res = self
            .get("https://spoc.buaa.edu.cn/spocnewht/cas")
            .send()
            .await?;
        if res.url().as_str().contains("https://sso.buaa.edu.cn/login") {
            return Err(Error::auth_expired(Location::Sso));
        }
        let mut query = res.url().query_pairs();
        let token = query
            .next()
            .and_then(|t| if t.0 == "token" { Some(t.1) } else { None })
            .ok_or_else(|| Error::server("[Spoc] Login failed. No token"))?;
        // 再次调用 next 获取 refreshToken, 但我们用不着, 使用我们自己的机制刷新登陆状态

        self.cred.set(Location::Spoc, token.to_string());
        Ok(())
    }
}
