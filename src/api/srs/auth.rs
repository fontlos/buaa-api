use crate::{Error, error::Location};

impl super::SrsApi {
    pub async fn login(&self) -> crate::Result<()> {
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && self.cred.load().sso.is_expired() {
            self.api::<crate::api::Core>().login().await?;
        }

        let url = "https://sso.buaa.edu.cn/login?service=https%3A%2F%2Fbyxk.buaa.edu.cn%2Fxsxk%2Fauth%2Fcas";
        // 获取 JSESSIONID
        let res = self.get(url).send().await?;
        // 未转跳就证明登录过期
        if res.url().as_str() == url {
            return Err(Error::LoginExpired(Location::Sso));
        }
        // 储存 token
        let cookie = self.cookies.load();
        match cookie.get("byxk.buaa.edu.cn", "/xsxk", "token") {
            Some(t) => {
                self.cred.update(|c| {
                    // TODO: 我们先默认十分钟过期, 待测试
                    c.srs_token.set(t.to_string(), 600);
                    // 刷新 SSO 时效
                    c.sso.refresh(5400);
                });
                Ok(())
            }
            None => Err(Error::ServerError("No Token".to_string())),
        }
    }
}
