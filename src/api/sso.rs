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
        // 获取登录页 execution 值
        let res = self.get("https://sso.buaa.edu.cn/login").send().await?;
        // 重定向到这里说明 Cookie 有效
        if res.url().as_str() == "https://uc.buaa.edu.cn/#/user/login" {
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
            .post("https://sso.buaa.edu.cn/login")
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
