use bytes::Bytes;
use serde::Serialize;

use crate::api::{Class, Sso};
use crate::error::Error;
use crate::{crypto, utils};

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
        let url = "https://iclass.buaa.edu.cn:8346/";
        let res = self.client.get(url).send().await?;

        // 整个这一次请求的意义存疑, 但也许是为了验证 loginName 是否有效
        let url = res.url().as_str().as_bytes();
        let session = utils::parse_by_tag(url, "loginName=", "")
            .ok_or_else(|| Error::server("No loginName found").with_label("Class"))?;
        // 使用 DES 加密 URL, 这是下一步请求的参数之一
        let cipher = crypto::des::Des::new(CLASS_DES_KEY);
        let url = cipher.encrypt_ecb(url);
        let url = crypto::bytes2hex(&url);
        let query = [("method", "html5GetPrivateUserInfo"), ("url", &url)];
        self.client
            .get("https://iclass.buaa.edu.cn:8346/wc/auth/html5GetPrivateUserInfo")
            .query(&query)
            .send()
            .await?;

        // 最终登录
        let query = [
            ("phone", session),
            ("password", ""),
            ("verificationType", "2"),
            ("verificationUrl", ""),
            ("userLevel", "1"),
        ];
        // 2025.12.28 学校后端 NGINX 改错了导致所有 /app/ 路径的 8346 端口被挂载到 /app/app/ 下了
        // 临时改成 8347 端口绕过
        // 如果以后不影响使用就保持这样
        // 包括 opt 模块的一些请求 URL 也是相同的处理
        // 很难想象能有这种错误发生
        let res = self
            .client
            .get("https://iclass.buaa.edu.cn:8347/app/user/login.action")
            .query(&query)
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
            None => Err(Error::server("Login failed. No token").with_label("Class")),
        }
    }

    /// Universal Request for ClassApi (Internal)
    ///
    /// **Note**: `token` parameter is already included
    pub(crate) async fn universal_request<P>(&self, url: &str, payload: &P) -> crate::Result<Bytes>
    where
        P: Serialize + ?Sized,
    {
        let cred = self.cred.load();
        if cred.is_expired::<Class>() {
            self.login().await?;
        }
        let token = cred.value::<Class>()?;

        // 因为双 token 机制, 我们暂时只是简单的将其拼在一起
        let (session, id) = token
            .split_once('@')
            .ok_or(Error::auth("Cannot split 'session' and 'id' token").with_label("Class"))?;

        // 在 URL 中硬编码 id
        let bytes = self
            .client
            .post(format!("{url}?id={id}"))
            .header("Sessionid", session)
            .query(&payload)
            .send()
            .await?
            .bytes()
            .await?;
        Ok(bytes)
    }
}
