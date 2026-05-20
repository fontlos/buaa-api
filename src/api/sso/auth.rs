use log::trace;

use crate::api::{Sso, Vpn};
use crate::error::Error;
use crate::utils;

impl super::SsoApi {
    /// # Login to SSO
    pub async fn login(&self) -> crate::Result<()> {
        let cred = self.cred.load();
        let login_url = "https://sso.buaa.edu.cn/login";
        let verify_url = "https://uc.buaa.edu.cn/";
        self.login_inner(login_url, verify_url).await?;
        cred.refresh::<Sso>();
        Ok(())
    }

    // 考虑到常用除了 class 都能直连, 那么 VPN 模式其实没那么重要, 让需要的情况自己调用
    /// # Login to SSO via VPN
    ///
    /// **Note**: Currently only for ClassAPI
    pub async fn login_vpn(&self) -> crate::Result<()> {
        let cred = self.cred.load();
        let login_url = "https://d.buaa.edu.cn/https/77726476706e69737468656265737421e3e44ed225256951300d8db9d6562d/login?service=https%3A%2F%2Fd.buaa.edu.cn%2Flogin%3Fcas_login%3Dtrue";
        let verify_url = "https://d.buaa.edu.cn/";
        self.login_inner(login_url, verify_url).await?;
        cred.refresh::<Vpn>();
        Ok(())
    }

    async fn login_inner(&self, login_url: &str, verify_url: &str) -> crate::Result<()> {
        let cred = self.cred.load();
        let un = cred.username()?;
        let pw = cred.password()?;
        // 获取登录页 execution 值
        let res = self.client.get(login_url).send().await?;
        // 重定向到这里说明 Cookie 有效, 但无法刷新
        if res.url().as_str() == verify_url {
            trace!("SSO still valid");
            return Ok(());
        }
        let bytes = res.bytes().await?;
        let execution = match utils::parse_by_tag(&bytes, "\"execution\" value=\"", "\"") {
            Some(s) => s,
            None => {
                return Err(Error::server("Login failed. No Execution Value").with_label("Sso"));
            }
        };
        let form = [
            ("username", un),
            ("password", pw),
            ("submit", "登录"),
            ("type", "username_password"),
            ("execution", execution),
            ("_eventId", "submit"),
        ];
        let res = self.client.post(login_url).form(&form).send().await?;
        // 只有状态码是有效信息
        let status = res.status();
        if status.is_success() {
            return Ok(());
        }
        // 当 "账号存在安全风险", "您的密码已过期或是已知的弱密码" 时尝试忽略风险继续登录
        let bytes = res.bytes().await?;
        if bytes
            .windows(b"continueForm".len())
            .any(|w| w == b"continueForm")
        {
            trace!(
                "SSO account has security risk, try to ignore and continue. Suggest change password."
            );
            let execution = match utils::parse_by_tag(&bytes, "\"execution\" value=\"", "\"") {
                Some(s) => s,
                None => {
                    return Err(Error::server(
                        "Ignore the risk and continue login failed. No Execution Value",
                    )
                    .with_label("Sso"));
                }
            };
            let form = [("execution", execution), ("_eventId", "ignoreAndContinue")];
            let res = self.client.post(login_url).form(&form).send().await?;
            if res.status().is_success() {
                return Ok(());
            }
        }
        let source = format!("Response Code: {}", status);
        Err(
            Error::server("Login failed. Maybe wrong username or password")
                .with_label("Sso")
                .with_source(source),
        )
    }
}
