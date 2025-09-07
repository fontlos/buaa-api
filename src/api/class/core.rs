use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::api::Location;
use crate::error::Error;
use crate::{crypto, utils};

use super::_ClassRes;

/// From the reverse analysis of JS
/// 2025.04.22
const CLASS_DES_KEY: &[u8] = b"Jyd#351*";

impl super::ClassApi {
    /// # Smart Classroom Login
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().sso.is_expired() {
            self.api::<crate::api::Sso>().login().await?;
        }

        // 获取 JSESSIONID
        let res = self.get("https://iclass.buaa.edu.cn:8346/").send().await?;

        // 整个这一次请求的意义存疑, 但也许是为了验证 loginName 是否有效
        // 2025.09.07 早期版本中 URL 末尾有 #/, 现在似乎去掉了
        let url = res.url().as_str();
        // 如果获取失败, 说明登录已过期, 则重新登录
        // 兼容性处理, 早期版本中 URL 末尾有 #/, 现在似乎去掉了
        let session = match url.find("loginName=") {
            Some(start) => {
                let mut v = &url[start + "loginName=".len()..];
                if let Some(stripped) = v.strip_suffix("#/") {
                    v = stripped;
                }
                v
            }
            None => return Err(Error::auth_expired(Location::Sso)),
        };

        // 使用 DES 加密 URL, 这是下一步请求的参数之一
        let cipher = crypto::des::Des::new(CLASS_DES_KEY).unwrap();
        let url = cipher.encrypt_ecb(url.as_bytes());
        let url = crypto::bytes2hex(&url);
        let params = [("method", "html5GetPrivateUserInfo"), ("url", &url)];
        self.get("https://iclass.buaa.edu.cn:8346/wc/auth/html5GetPrivateUserInfo")
            .query(&params)
            .send()
            .await?;

        let params = [
            ("phone", session),
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

        // 2025.09.07 后端更新, ClassApi 使用了双 token
        // 因为其他 Api 没有这样的需要, 所以我们直接在这里把它们拼起来
        // 至于具体使用见下面通用请求方法
        // 尽管 res 里面也有 session, 但毕竟上面就解析出来使用过了, 这里就不解析了直接切割字符串
        let res = res.text().await?;
        match utils::get_value_by_lable(&res, "\"id\":\"", "\"") {
            Some(id) => {
                self.cred.set(Location::Class, format!("{session}@{id}"));
                Ok(())
            }
            None => Err(Error::server("[Class] Login failed. No token")),
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
        // 因为双 token 机制, 我们暂时只是简单的将其拼在一起
        let (session, id) = token.split_once('@').unwrap();

        // 在 URL 中硬编码 id
        let res = self
            .post(format!("{url}?id={id}"))
            .header("Sessionid", session)
            .query(&query)
            .send()
            .await?
            .bytes()
            .await?;
        let res = serde_json::from_slice::<_ClassRes<T>>(&res)?;

        if res.status != "0" {
            return Err(Error::server(format!(
                "[Class] Response: {}",
                res.msg.unwrap_or("Unknown error".to_string())
            )));
        }

        match res.result {
            Some(r) => Ok(r),
            None => Err(Error::server("[Class] No result")),
        }
    }
}
