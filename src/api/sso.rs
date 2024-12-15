use crate::{Context, Error};

use crate::utils;

impl Context {
    /// # SSO Login
    /// This is the most important method and should be called first <br>
    /// This method is used to login to the SSO system, and the login information will be saved in the cookie <br>
    /// If your login information expires, you should also re-call this function to refresh the cookie
    /// ```rust
    /// use buaa::Context;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let context = Context::new();
    ///     context.set_account("username", "password")
    ///     context.with_cookies("cookie.json");
    ///     context.login().await.unwrap();
    ///
    ///     // do something
    ///
    ///     context.save();
    /// }
    /// ```
    pub async fn login(&self) -> crate::Result<()> {
        let login_url = "https://sso.buaa.edu.cn/login";
        let verify_url = "https://uc.buaa.edu.cn/#/user/login";
        self.sso_login_internal(login_url, verify_url).await
    }

    pub async fn login_vpn(&self) -> crate::Result<()> {
        let login_url = "https://d.buaa.edu.cn/https/77726476706e69737468656265737421e3e44ed225256951300d8db9d6562d/login?service=https%3A%2F%2Fd.buaa.edu.cn%2Flogin%3Fcas_login%3Dtrue";
        let verify_url = "https://d.buaa.edu.cn/";
        self.sso_login_internal(login_url, verify_url).await
    }

    async fn sso_login_internal(
        &self,
        login_url: &str,
        verify_url: &str,
    ) -> crate::Result<()> {
        let config = self.config.read().unwrap();
        let un = match config.username.as_ref() {
            Some(s) => s,
            None => return Err(Error::LoginError("No Username".to_string()))?,
        };
        let pw = match config.password.as_ref() {
            Some(s) => s,
            None => return Err(Error::LoginError("No Password".to_string()))?,
        };
        // 获取登录页 execution 值
        let res = self.get(login_url).send().await?;
        // 重定向到这里说明 Cookie 有效
        if res.url().as_str() == verify_url {
            return Ok(());
        }
        let html = res.text().await?;
        let execution = match utils::get_value_by_lable(&html, "\"execution\" value=\"", "\"") {
            Some(s) => s,
            None => {
                return Err(Error::LoginError("No Execution Value".to_string()));
            }
        };
        let form = [
            ("username", &un[..]),
            ("password", &pw[..]),
            ("submit", "登录"),
            ("type", "username_password"),
            ("execution", &execution),
            ("_eventId", "submit"),
        ];
        let res = self.post(login_url).form(&form).send().await?;
        if res.status().as_u16() == 200 {
            Ok(())
        } else {
            Err(Error::LoginError(String::from(
                "Maybe wrong username or password",
            )))
        }
    }
}
