use crate::Error;

impl super::SpocAPI {
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
            return Err(Error::LoginExpired("SSO Expired".to_string()));
        }
        let mut query = res.url().query_pairs();
        let token = match query.next() {
            Some((key, value)) => {
                if key == "token" {
                    value
                } else {
                    return Err(Error::LoginError("No Token".to_string()));
                }
            }
            None => return Err(Error::LoginError("No Token".to_string())),
        };
        // 用来刷新 Token 的 Refresh Token, 但我们用不着
        // let _refresh_token = match query.next() {
        //     Some((key, value)) => {
        //         if key == "refreshToken" {
        //             value
        //         } else {
        //             return Err(SessionError::LoginError("No Refresh Token".to_string()));
        //         }
        //     }
        //     None => return Err(SessionError::LoginError("No Refresh Token".to_string())),
        // };
        self.cred.update(|c| {
            // TODO: 至少 14 小时, 待测试
            c.spoc_token.set(token.to_string(), 50400);
            // 刷新 SSO 时效
            c.sso.refresh(5400);
        });
        Ok(())
    }
}
