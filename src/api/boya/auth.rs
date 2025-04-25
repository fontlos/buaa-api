use crate::Error;

impl super::BoyaAPI {
    /// # Boya Login
    pub async fn login(&self) -> crate::Result<()> {
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
            self.config.update(|c| {
                c.boya_token = Some(token.1.to_string());
            });
            return Ok(());
        } else {
            return Err(Error::LoginError("No Token".to_string()));
        }
    }

    pub async fn login_vpn(&self) -> crate::Result<()> {
        let url = "https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/cas/login";
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
            self.config.update(|c| {
                c.boya_token = Some(token.1.to_string());
            });
            return Ok(());
        } else {
            return Err(Error::LoginError("No Token".to_string()));
        }
    }
}
