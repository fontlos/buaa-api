use cookie_store::{Cookie, CookieStore};
use reqwest::Client;
use reqwest_cookie_store::CookieStoreMutex;
use thiserror::Error;

use std::fs::{self, File};
use std::io::BufReader;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use crate::utils;

/// This is the core of this crate, it is used to store cookies and send requests </br>
/// The prefix of most API names is derived from the fourth-level domain name of the corresponding domain name
#[derive(Debug)]
pub struct Session {
    client: Client,
    cookie_path: Option<PathBuf>,
    cookie_store: Arc<CookieStoreMutex>,
}

#[derive(Debug, Error)]
pub enum SessionError{
    #[error("Cookie Error")]
    CookieError,
    #[error("No Execution Value. BUT this is usually not an error, it means the cookie is still valid, causes redirects to a post-landing page, so you can't get the execution value")]
    NoExecutionValue,
    #[error("Request Error")]
    RequestError,
    #[error("Login Error: {0}")]
    LoginError(String),
}

impl Session {
    /// Create a new session in memory, if you call `save` method, it will save cookies to `cookies.json` defaultly
    /// ```rust
    /// use buaa::Session;
    ///
    /// fn main() {
    ///     let session = Session::new_in_memory();
    ///     // if you call `save` method, it will save cookies to `cookies.json` defaultly
    ///     // session.save();
    /// }
    /// ```
    pub fn new_in_memory() -> Self {
        let cookie_store = Arc::new(CookieStoreMutex::new(CookieStore::default()));

        let client = Client::builder()
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();

        Session {
            client,
            cookie_path: None,
            cookie_store,
        }
    }
    /// Create a new session in file, if the file is not exist, it will create a new one, but It won't be saved until you call `save` method
    /// ```rust
    /// use buaa::Session;
    ///
    /// fn main() {
    ///     let session = Session::new_in_file("path_to_cookies.json");
    ///     session.save();
    /// }
    /// ```
    pub fn new_in_file(path: &str) -> Self {
        let path = PathBuf::from(path);
        let cookie_store = match File::open(&path) {
            Ok(f) => CookieStore::load_all(
                BufReader::new(f),
                |s| serde_json::from_str::<Cookie>(s),
            ).unwrap(),
            Err(_) => CookieStore::default(),
        };

        let cookie_store = Arc::new(CookieStoreMutex::new(cookie_store));

        let client = Client::builder()
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();

        Session {
            client,
            cookie_path: Some(path),
            cookie_store,
        }
    }
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
        let execution = if res.status().is_success() {
            let html = res.text().await.unwrap();
            match utils::get_value_by_lable(&html, "\"execution\" value=\"", "\"") {
                Some(s) => s,
                // TODO 加入 Session 构造器后需要重构这部分
                // 通常情况下这不是一个错误, 所以只有当使用 `new_in_memory` 时会触发这个错误
                // 找不到 execution 值，说明登录请求自动重定向到登陆后的页面, 证明当前 Cookie 有效
                // 当 Cookie 无效时会重定向到登录 URL, 此时可以刷新 Cookie
                // 等到支持 Session 构造器时, 可以加入对客户端是否自动重定向的配置, 这时可以更好的检测问题
                None => if self.cookie_path.is_none() {
                    return Err(SessionError::NoExecutionValue)
                } else {
                    return Ok(())
                }
            }
        } else {
            return Err(SessionError::RequestError);
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

    /// save cookies manually
    pub fn save(&mut self) {
        let path = match &self.cookie_path {
            Some(p) => p.to_str().unwrap(),
            None => {
                "cookies.json"
            },
        };
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open cookie file: {:?}", e);
                return
            },
        };
        let store = self.cookie_store.lock().unwrap();
        if let Err(e) = store.save_incl_expired_and_nonpersistent(&mut file, |s| serde_json::to_string(s)) {
            eprintln!("Failed to save cookie store: {}", e);
        }
    }
}

impl Deref for Session {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[tokio::test]
async fn test_login() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.login(&username, &password).await.unwrap();

    session.uc_login().await.unwrap();
    let state = session.uc_get_state().await.unwrap();
    println!("{}", state);

    session.save();
}
