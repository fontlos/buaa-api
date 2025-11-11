use log::trace;

use crate::api::Sso;
use crate::error::Error;
use crate::utils;

impl super::SsoApi {
    /// # Login to SSO
    pub async fn login(&self) -> crate::Result<()> {
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // "https://d.buaa.edu.cn/https/77726476706e69737468656265737421e3e44ed225256951300d8db9d6562d/login?service=https%3A%2F%2Fd.buaa.edu.cn%2Flogin%3Fcas_login%3Dtrue";
        // "https://d.buaa.edu.cn/";
        let login_url = "https://sso.buaa.edu.cn/login";
        let verify_url = "https://uc.buaa.edu.cn/";
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
        let status = res.status().as_u16();
        if status == 200 {
            cred.refresh::<Sso>();
            Ok(())
        } else {
            let source = format!("Response Code: {}", status);
            Err(
                Error::server("Login failed. Maybe wrong username or password")
                    .with_label("Sso")
                    .with_source(source),
            )
        }
    }
}
