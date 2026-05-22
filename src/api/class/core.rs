use bytes::Bytes;
use serde::Serialize;

use crate::api::{Class, Sso, Vpn};
use crate::error::Error;
use crate::utils;

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
        // 防止共同竞争触发 423 Locked 错误
        if !is_vpn && self.cred.load().is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        let session = self.cred.load().username()?;
        let query = [
            ("phone", session),
            ("password", ""),
            ("verificationType", "2"),
            ("verificationUrl", ""),
            ("userLevel", "1"),
        ];
        // 2025.12.28 学校后端 NGINX 改错了导致所有 /app/ 路径的 8346 端口被挂载到 /app/app/ 下了
        // 临时改成 8347 端口绕过, 如果以后不影响使用就保持这样, 包括 opt 模块的一些请求 URL 也是相同的处理
        // 很难想象能有这种错误发生
        let res = self
            .client
            .get(Self::url(is_vpn, 8346, "app/user/login.action"))
            .query(&query)
            .send()
            .await?
            .bytes()
            .await?;

        // 2026.05.20 所以根本不是双 Token, 而是原来的 Session 作废了直接用用户名代替了吗??
        // 之前需要从 https://iclass.buaa.edu.cn:8346 重定向 URL 得到 loginName 作为 Session
        // 使用 DES ECB (Key = Jyd#351*) 加密重定向的 URL 作为参数
        // 对 Path: wc/auth/html5GetPrivateUserInfo 发起 method=html5GetPrivateUserInfo&url={ENCODED_URL} 的请求
        // 然后所有请求都通过 'Sessionid' Header 携带
        // 虽然下面也能从 'sessionId' 解析出那个值, 但我不尊重学校, 既然用不到 Session 了干脆不解析了
        // 反正上面都用 用户名 了, 后续请求也不差这点
        match utils::parse_by_tag(&res, "\"id\":\"", "\"") {
            Some(id) => {
                self.cred.update(|s| {
                    s.update::<Class>(id.to_string());
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

        let is_vpn = !utils::net::is_on_campus_network();
        let url = Self::url(is_vpn, port, path);

        // 在 URL 中硬编码 id
        let bytes = self
            .client
            .post(format!("{url}?id={token}"))
            .query(&payload)
            .send()
            .await?
            .bytes()
            .await?;
        Ok(bytes)
    }
}
