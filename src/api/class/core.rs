use bytes::Bytes;
use serde::Serialize;

use crate::api::{Class, Sso, Vpn};
use crate::error::Error;
use crate::{crypto, utils};

/// From the reverse analysis of JS
/// 2025.04.22
const CLASS_DES_KEY: &[u8] = b"Jyd#351*";

impl super::ClassApi {
    fn url(is_vpn: bool, port: u16, path: &str) -> String {
        if is_vpn {
            format!(
                "https://d.buaa.edu.cn/https-8347/77726476706e69737468656265737421f9f44d9d342326526b0988e29d51367ba018/{path}"
            )
        } else {
            format!("https://iclass.buaa.edu.cn:{port}/{path}")
        }
    }

    /// # Login to ClassApi
    pub async fn login(&self) -> crate::Result<()> {
        // 注意, Class 登陆状态是可随意复写的, 调用一次复写一次
        // 再加上 VPN 模式和普通模式的 Cred 和 Cookie 是可以互相用的
        // 也就是说唯一需要仔细处理的地方就是 SsoAPI::login_vpn
        // 确保在需要的时刻调用它以免 VPN 模式下 Class 登录流程失败

        let is_vpn = !utils::net::is_on_campus_network();
        // 只在需要时刷新 VPN 的 SSO
        if is_vpn && self.cred.load().is_expired::<Vpn>() {
            self.api::<Sso>().login_vpn().await?;
        }
        if self.cred.load().is_expired::<Sso>() {
            // 在任何情况下, 刷新 SSO 都是值得的
            self.api::<Sso>().login().await?;
        }

        // 获取 JSESSIONID
        let url = Self::url(is_vpn, 8346, "");
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
            .get(Self::url(is_vpn, 8346, "wc/auth/html5GetPrivateUserInfo"))
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
            .get(Self::url(is_vpn, 8346, "app/user/login.action"))
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
    pub(crate) async fn universal_request<P>(
        &self,
        port: u16,
        path: &str,
        payload: &P,
    ) -> crate::Result<Bytes>
    where
        P: Serialize + ?Sized,
    {
        let cred = self.cred.load();
        // 注意, 这里无需特殊处理, VPN 模式和正常模式的 Cred 和 Cookie 是可以互相用的
        // 也就是说唯一需要仔细处理的地方就是 ClassAPI::login 函数自己
        if cred.is_expired::<Class>() {
            self.login().await?;
        }
        let token = cred.value::<Class>()?;

        // 因为双 token 机制, 我们暂时只是简单的将其拼在一起
        let (session, id) = token
            .split_once('@')
            .ok_or(Error::auth("Cannot split 'session' and 'id' token").with_label("Class"))?;

        let is_vpn = !utils::net::is_on_campus_network();
        let url = Self::url(is_vpn, port, path);

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
