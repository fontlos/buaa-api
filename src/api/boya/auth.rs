use crate::error::{Error, Location};

impl super::BoyaApi {
    /// # Boya Login
    pub async fn login(&self) -> crate::Result<()> {
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && self.cred.load().sso.is_expired() {
            self.api::<crate::api::Core>().login().await?;
        }

        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/cas/login
        let url = "https://sso.buaa.edu.cn/login?noAutoRedirect=true&service=https%3A%2F%2Fbykc.buaa.edu.cn%2Fsscv%2Fcas%2Flogin";
        // 获取 JSESSIONID
        let res = self.get(url).send().await?;
        // 未转跳就证明登录过期
        if res.url().as_str() == url {
            return Err(Error::auth_expired(Location::Sso));
        }
        let mut query = res.url().query_pairs();
        let token = match query.next() {
            Some(t) => t,
            None => return Err(Error::ServerError("No Token".to_string())),
        };
        if token.0 == "token" {
            self.cred.update(|c| {
                // 经验证 15 分钟内过期, 我们这里用 10 分钟
                c.boya_token.set(token.1.to_string(), 600);
                // 刷新 SSO 时效
                c.sso.refresh(5400);
            });
            Ok(())
        } else {
            Err(Error::ServerError("No Token".to_string()))
        }
    }
}
