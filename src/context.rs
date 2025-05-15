use reqwest::{
    Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};

use std::marker::PhantomData;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

use crate::{api::SSO, cell::AtomicCell};
use crate::store::cookies::AtomicCookieStore;
use crate::store::cred::CredentialStore;

/// This is the core of this crate, it is used to store cookies and send requests <br>
pub struct Context<G = SSO> {
    pub(crate) client: Client,
    pub(crate) cookies: Arc<AtomicCookieStore>,
    pub(crate) cred: AtomicCell<CredentialStore>,
    _marker: PhantomData<G>,
}

impl Context {
    /// Initialize the `Context`
    /// ```rust
    /// use buaa::Context;
    ///
    /// fn main() {
    ///     let mut context = Context::new();
    ///     // Set account
    ///     context.set_account("username", "password");
    ///     // Login to context
    ///     context.login().await.unwrap();
    /// }
    /// ```
    pub fn new() -> Context<SSO> {
        let cookies = Arc::new(AtomicCookieStore::default());
        let mut header = HeaderMap::new();
        header.insert(HeaderName::from_bytes(b"User-Agent").unwrap(), HeaderValue::from_bytes(b"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0").unwrap());

        let client = Client::builder()
            .default_headers(header)
            .cookie_provider(cookies.clone())
            .build()
            .unwrap();

        Context {
            client,
            cookies,
            cred: AtomicCell::new(CredentialStore::default()),
            _marker: PhantomData,
        }
    }

    /// Initialize with authentication data (credentials and cookies) from specified directory.
    ///
    /// This will attempt to load:
    /// - Cookies from `./dir/cookies.json`
    /// - Credentials from `./dir/cred.json`
    ///
    /// If either file doesn't exist or fails to load, default values will be used instead.
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn with_auth<P: AsRef<Path>>(dir: P) -> Context<SSO> {
        let cookies_path = dir.as_ref().join("cookies.json");
        let cookies = Arc::new(AtomicCookieStore::new(
            AtomicCookieStore::from_file(cookies_path),
        ));
        let cred_path = dir.as_ref().join("cred.json");
        let cred = CredentialStore::from_file(cred_path);

        let mut header = HeaderMap::new();
        header.insert(HeaderName::from_bytes(b"User-Agent").unwrap(), HeaderValue::from_bytes(b"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0").unwrap());

        let client = Client::builder()
            .default_headers(header)
            .cookie_provider(cookies.clone())
            .build()
            .unwrap();

        Context {
            client,
            cookies,
            cred: AtomicCell::new(cred),
            _marker: PhantomData,
        }
    }

    pub fn set_account(&self, username: &str, password: &str) {
        self.cred.update(|c| {
            c.username = Some(username.to_string());
            c.password = Some(password.to_string());
        });
    }

    pub fn set_username(&self, username: &str) {
        self.cred.update(|c| {
            c.username = Some(username.to_string());
        });
    }

    pub fn set_password(&self, password: &str) {
        self.cred.update(|c| {
            c.password = Some(password.to_string());
        });
    }

    pub fn set_cookies(&self, cookies: cookie_store::CookieStore) {
        // 这在理论上是安全的
        // 因为没有理由在多线程中频繁切换 cookie
        // 如果有并发需求应该创建多个 Context 而不是切换这个
        // 它唯一的作用就是用于切换账号. 这能保证不会有卡在中间执行的请求
        // 其他 load 方法的生命周期局限在自己的函数中
        self.cookies.store(cookies);
    }

    pub fn set_cred(&self, cred: CredentialStore) {
        self.cred.store(cred);
    }

    pub fn get_cookies(&self) -> &AtomicCookieStore {
        &self.cookies
    }

    pub fn get_cred(&self) -> &CredentialStore {
        self.cred.load()
    }

    /// Load authentication data (credentials and cookies) from specified directory.
    ///
    /// This will attempt to load:
    /// - Cookies from `./dir/cookies.json`
    /// - Credentials from `./dir/cred.json`
    ///
    /// If either file doesn't exist or fails to load, default values will be used instead.
    /// For more precise control over loading behavior, you can manually construct and set the auth data:
    /// ```
    /// let context = Context::new();
    /// let cookie = AtomicCookieStore::from_file(path);
    /// context.set_cookies(cookie);
    /// ```
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn load_auth<P: AsRef<Path>>(&self, dir: P) {
        let cookies_path = dir.as_ref().join("cookies.json");
        let cookies = AtomicCookieStore::from_file(cookies_path);
        let cred_path = dir.as_ref().join("cred.json");
        let cred = CredentialStore::from_file(cred_path);
        self.set_cookies(cookies);
        self.set_cred(cred);
    }

    /// Save authentication data (credentials and cookies) to specified directory.
    ///
    /// This will attempt to save:
    /// - Cookies to `./dir/cookies.json`
    /// - Credentials to `./dir/cred.json`
    ///
    /// For more precise control over loading behavior, you can manually construct and set the auth data:
    /// ```
    /// let cookies = context.get_cookies();
    /// cookies.to_file(path);
    /// ```
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn save_auth<P: AsRef<Path>>(&self, dir: P) {
        let cookies_path = dir.as_ref().join("cookies.json");
        let cred_path = dir.as_ref().join("cred.json");
        self.get_cookies().to_file(cookies_path);
        self.get_cred().to_file(cred_path);
    }
}

impl<G> Deref for Context<G> {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
