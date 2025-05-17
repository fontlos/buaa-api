use crate::{store::cred::CredentialItem, Error, utils};

impl super::BoyaAPI {
    /// # Boya Login
    pub async fn login(&self) -> crate::Result<()> {
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/cas/login
        let url = "https://sso.buaa.edu.cn/login?noAutoRedirect=true&service=https%3A%2F%2Fbykc.buaa.edu.cn%2Fsscv%2Fcas%2Flogin";
        // 获取 JSESSIONID
        let res = self.get(url).send().await?;
        // 未转跳就证明登录过期
        if res.url().as_str() == url {
            return Err(Error::LoginExpired("SSO Expired".to_string()));
        }
        let mut query = res.url().query_pairs();
        let token = match query.next() {
            Some(t) => t,
            None => return Err(Error::LoginError("No Token".to_string())),
        };
        if token.0 == "token" {
            self.cred.update(|c| {
                c.boya_token = Some(CredentialItem{
                    value: token.1.to_string(),
                    // 经验证十五分钟内过期, 我们这里用十分钟
                    expiration: utils::get_time_secs() + 600,
                });
            });
            return Ok(());
        } else {
            return Err(Error::LoginError("No Token".to_string()));
        }
    }

    // 仅在启用自动刷新策略并且登录过期时才会刷新
    // 检测是否过期是我们的保守策略, 即超出我们的预期, 并不保证是否一定是登录过期
    pub(crate) fn need_refresh(&self) -> bool {
        // 如果根本不启用自动刷新策略, 那么就不需要刷新
        if !self.policy.load().is_auto() {
            return false;
        };
        // 如果能拿到, 那看看过没过期
        if let Some(c) = &self.cred.load().boya_token {
            // 过期了就刷新
            return c.is_expired()
        } else {
            // 如果没有, 那就刷新
            return true
        };
    }
}
