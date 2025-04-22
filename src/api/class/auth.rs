use crate::{Error, crypto, utils};

use super::_ClassLogin;

impl super::ClassAPI {
    /// # Smart Classroom Login
    pub async fn login(&self) -> crate::Result<()> {
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
        let url = crypto::des::des_encrypt(url, crate::consts::CLASS_DES_KEY);
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
                let mut config = self.config.write().unwrap();
                config.class_token = Some(res.result.id);
                Ok(())
            }
            Err(_) => Err(Error::LoginError(format!(
                "Smart Classroom Login Failed: {}",
                res
            ))),
        }
    }
}
