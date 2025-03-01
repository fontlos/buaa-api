use super::ElectiveAPI;
use crate::Error;

impl ElectiveAPI {
    pub async fn login(&self) -> crate::Result<()> {
        let url = "https://sso.buaa.edu.cn/login?service=https%3A%2F%2Fbyxk.buaa.edu.cn%2Fxsxk%2Fauth%2Fcas";
        // 获取 JSESSIONID
        let res = self.get(url).send().await?;
        // 未转跳就证明登录过期
        if res.url().as_str() == url {
            return Err(Error::LoginExpired("SSO Expired".to_string()));
        }
        // 储存 token
        let cookie = self.cookies.lock().unwrap();
        match cookie.get("byxk.buaa.edu.cn", "/xsxk", "token") {
            Some(t) => {
                let mut config = self.config.write().unwrap();
                config.elective_token = Some(t.value().to_string());
                return Ok(());
            }
            None => return Err(Error::LoginError("No Token".to_string())),
        }
    }
}
