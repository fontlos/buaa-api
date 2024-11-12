use crate::{Session, SessionError};

use crate::utils;

impl Session {
    /// # SSO Login
    /// This is the most important method and should be called first, so it named `login` directly </br>
    /// This method is used to login to the SSO system, and the login information will be saved in the cookie </br>
    /// If your login information expires, you should also re-call this function to refresh the cookie
    /// ```rust
    /// use buaa::Session;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut session = Session::new_in_file("cookie.json");
    ///
    ///     session.login("username", "password").await.unwrap();
    ///
    ///     // do something
    ///
    ///     session.save();
    /// }
    /// ```
    pub async fn login(&self, un:&str, pw: &str) -> Result<(), SessionError> {
        // 获取登录页 execution 值
        let res = self.get("https://sso.buaa.edu.cn")
            .send()
            .await
            .unwrap();
        let html = res.text().await.unwrap();
        let execution = match utils::get_value_by_lable(&html, "\"execution\" value=\"", "\"") {
            Some(s) => s,
            // TODO 加入 Session 构造器后需要重构这部分
            // 通常情况下这不是一个错误, 所以只有当使用 `new_in_memory` 时会触发这个错误
            // 找不到 execution 值，说明登录请求自动重定向到登陆后的页面, 证明当前 Cookie 有效
            // 当 Cookie 无效时会重定向到登录 URL, 此时可以刷新 Cookie
            // 等到支持 Session 构造器时, 可以加入对客户端是否自动重定向的配置, 这时可以更好的检测问题
            None => if !self.have_cookie_path() {
                return Err(SessionError::NoExecutionValue)
            } else {
                return Ok(())
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
        let res = self.post("https://sso.buaa.edu.cn/login")
            .form(&form)
            .send()
            .await
            .unwrap();
        if res.status().as_u16() == 200 {
            Ok(())
        } else {
            Err(SessionError::LoginError(String::from("SSO Username or Password Error")))
        }
    }
}

