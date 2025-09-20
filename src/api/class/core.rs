use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::api::{Class, Sso};
use crate::error::Error;
use crate::{crypto, utils};

use super::_ClassRes;

/// From the reverse analysis of JS
/// 2025.04.22
const CLASS_DES_KEY: &[u8] = b"Jyd#351*";

impl super::ClassApi {
    /// # Login to ClassApi
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        // 获取 JSESSIONID
        let res = self.get("https://iclass.buaa.edu.cn:8346/").send().await?;

        // 整个这一次请求的意义存疑, 但也许是为了验证 loginName 是否有效
        let url = res.url().as_str().as_bytes();
        let session = match utils::parse_by_tag(url, "loginName=", "") {
            Some(s) => s,
            // 理论上这是个不该发生的错误
            None => return Err(Error::server("[Class] No LoginName found")),
        };

        // 使用 DES 加密 URL, 这是下一步请求的参数之一
        let cipher = crypto::des::Des::new(CLASS_DES_KEY).unwrap();
        let url = cipher.encrypt_ecb(url);
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
            .await?
            .bytes()
            .await?;

        // 2025.09.07 后端更新, ClassApi 使用了双 token
        // 因为其他 Api 没有这样的需要, 所以我们直接在这里把它们拼起来
        // 至于具体使用见下面通用请求方法
        // 尽管 res 里面也有 session, 但毕竟上面就解析出来使用过了, 这里就不解析了直接切割字符串
        match utils::parse_by_tag(&res, "\"id\":\"", "\"") {
            Some(id) => {
                self.cred.update(|s| {
                    s.update::<Class>(format!("{session}@{id}"));
                });
                Ok(())
            }
            None => Err(Error::server("[Class] Login failed. No token")),
        }
    }

    // 内部方法不公开
    /// Universal Request for ClassApi
    ///
    /// **Note**: `token` parameter is already included
    pub(crate) async fn universal_request<Q, T>(&self, url: &str, query: &Q) -> crate::Result<T>
    where
        Q: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let cred = self.cred.load();
        if cred.is_expired::<Class>() {
            self.login().await?;
        }
        let token = cred.value::<Class>()?;

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
                res.msg.as_deref().unwrap_or("Unknown error")
            )));
        }

        match res.result {
            Some(r) => Ok(r),
            None => Err(Error::server("[Class] No result")),
        }
    }
}
