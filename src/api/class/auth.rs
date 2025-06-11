use crate::{Error, crypto, utils};

use super::_ClassLogin;

impl super::ClassAPI {
    /// # Smart Classroom Login
    pub async fn login(&self) -> crate::Result<()> {
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && self.cred.load().sso.is_expired() {
            self.api::<crate::api::Core>().login().await?;
        }

        // 获取 JSESSIONID
        let res = self.get("https://iclass.buaa.edu.cn:8346/").send().await?;

        // 整个这一次请求的意义存疑, 但也许是为了验证 loginName 是否有效
        let url = res.url().as_str();
        // 如果获取失败, 说明登录已过期, 则重新登录
        let login_name = match utils::get_value_by_lable(url, "loginName=", "#/") {
            Some(v) => v,
            None => return Err(Error::LoginExpired("SSO Login Expired".to_string())),
        };
        let url = &url[..url.len() - 2];
        // 使用 DES 加密 URL, 这是下一步请求的参数之一
        let url = crypto::des::des_encrypt(url.as_bytes(), crate::consts::CLASS_DES_KEY);
        let params = [("method", "html5GetPrivateUserInfo"), ("url", &url)];
        self.get("https://iclass.buaa.edu.cn:8346/wc/auth/html5GetPrivateUserInfo")
            .query(&params)
            .send()
            .await?;

        let params = [
            ("phone", login_name),
            ("password", ""),
            ("verificationType", "2"),
            ("verificationUrl", ""),
            ("userLevel", "1"),
        ];
        let res = self
            .get("https://iclass.buaa.edu.cn:8346/app/user/login.action")
            .query(&params)
            .send()
            .await?;
        let res = res.text().await?;
        match serde_json::from_str::<_ClassLogin>(&res) {
            Ok(res) => {
                self.cred.update(|c| {
                    // 至少 7 天, 但即使更多对我们也用处不大了, 也许以后有时间我会测一测极限时间
                    c.class_token.set(res.result.id, 604800);
                    // 刷新 SSO 时效
                    c.sso.refresh(5400);
                });
                Ok(())
            }
            Err(_) => Err(Error::LoginError(format!(
                "Smart Classroom Login Failed: {res}"
            ))),
        }
    }
}
