use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::api::Location;
use crate::error::Error;
use crate::{crypto, utils};

use super::_ClassLogin;

/// From the reverse analysis of JS
/// 2025.04.22
pub const CLASS_DES_KEY: &[u8] = b"Jyd#351*";

impl super::ClassApi {
    /// # Smart Classroom Login
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().sso.is_expired() {
            self.api::<crate::api::Sso>().login().await?;
        }

        // 获取 JSESSIONID
        let res = self.get("https://iclass.buaa.edu.cn:8346/").send().await?;

        // 整个这一次请求的意义存疑, 但也许是为了验证 loginName 是否有效
        let url = res.url().as_str();
        // 如果获取失败, 说明登录已过期, 则重新登录
        let login_name = match utils::get_value_by_lable(url, "loginName=", "#/") {
            Some(v) => v,
            None => return Err(Error::auth_expired(Location::Sso)),
        };
        // 去掉最后的 #/
        let url = &url[..url.len() - 2];
        // 使用 DES 加密 URL, 这是下一步请求的参数之一
        let url = crypto::des::des_encrypt(url.as_bytes(), CLASS_DES_KEY);
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
        // TODO: 到时候还是用字符串匹配吧, 不关心其他错误
        let res = res.text().await?;
        match serde_json::from_str::<_ClassLogin>(&res) {
            Ok(res) => {
                self.cred.set(Location::Class, res.result.id);
                Ok(())
            }
            Err(_) => Err(Error::server("[Class] Login failed. No token")),
        }
    }

    // 似乎没有什么公开的必要
    /// Class Universal Request API
    ///
    /// **Note**: `token` parameter is already included
    pub(crate) async fn universal_request<Q, T>(&self, url: &str, query: &Q) -> crate::Result<T>
    where
        Q: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        if self.cred.load().class_token.is_expired() {
            self.login().await?;
        }
        let cred = self.cred.load();
        let token = match cred.class_token.value() {
            Some(t) => t,
            None => {
                return Err(Error::auth_expired(Location::Class));
            }
        };
        // 在 URL 中硬编码 token
        let res = self
            .post(format!("{url}?id={token}"))
            .query(&query)
            .send()
            .await?
            .json::<T>()
            .await?;
        Ok(res)
    }
}
