use crate::{store::cred::CredentialItem, Error, utils};

impl super::SrsAPI {
    pub async fn login(&self) -> crate::Result<()> {
        let url = "https://sso.buaa.edu.cn/login?service=https%3A%2F%2Fbyxk.buaa.edu.cn%2Fxsxk%2Fauth%2Fcas";
        // 获取 JSESSIONID
        let res = self.get(url).send().await?;
        // 未转跳就证明登录过期
        if res.url().as_str() == url {
            return Err(Error::LoginExpired("SSO Expired".to_string()));
        }
        // 储存 token
        let cookie = self.cookies.load();
        match cookie.get("byxk.buaa.edu.cn", "/xsxk", "token") {
            Some(t) => {
                self.cred.update(|c| {
                    c.srs_token = Some(CredentialItem {
                        value: t.to_string(),
                        // TODO: 我们先默认十分钟过期, 待测试
                        expiration: utils::get_time_secs() + 600,
                    });
                });
                return Ok(());
            }
            None => return Err(Error::LoginError("No Token".to_string())),
        }
    }
}
