use crate::{Session, SessionError};

use crate::utils;

impl Session {
    /// # SSO Login
    /// This is the most important method and should be called first <br>
    /// This method is used to login to the SSO system, and the login information will be saved in the cookie <br>
    /// If your login information expires, you should also re-call this function to refresh the cookie
    /// ```rust
    /// use buaa::Session;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut session = Session::new_in_file("cookie.json");
    ///
    ///     session.sso_login("username", "password").await.unwrap();
    ///
    ///     // do something
    ///
    ///     session.save();
    /// }
    /// ```
    pub async fn sso_login(&self, un: &str, pw: &str) -> Result<(), SessionError> {
        let config = self.config.read().unwrap();
        let (login_url,  verify_url) = if config.vpn {
            ("https://d.buaa.edu.cn/https/77726476706e69737468656265737421e3e44ed225256951300d8db9d6562d/login?service=https%3A%2F%2Fd.buaa.edu.cn%2Flogin%3Fcas_login%3Dtrue", "https://d.buaa.edu.cn/")
        } else {
            ("https://sso.buaa.edu.cn/login", "https://uc.buaa.edu.cn/#/user/login")
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
                return Err(SessionError::LoginError("No Execution Value".to_string()));
            }
        };
        let form = [
            ("username", un),
            ("password", pw),
            ("submit", "登录"),
            ("type", "username_password"),
            ("execution", &execution),
            ("_eventId", "submit"),
        ];
        let res = self
            .post(login_url)
            .form(&form)
            .send()
            .await?;
        if res.status().as_u16() == 200 {
            Ok(())
        } else {
            Err(SessionError::LoginError(String::from(
                "Maybe wrong username or password",
            )))
        }
    }
}
