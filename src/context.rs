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

    /// Load cookies file to set Session cookies and set `cookie_path`, if the path is not exist, it will create a new file, but It won't be saved until you call `save` method
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn with_cookies<P: AsRef<Path>>(&self, path: P) {
        let cookie = AtomicCookieStore::from_file(path);
        self.set_cookies(cookie);
    }

    /// Load config from path, if the path is not exist or parse failed, it will return a default one
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn with_cred<P: AsRef<Path>>(&self, path: P) {
        let cred = CredentialStore::from_file(path);
        self.set_cred(cred);
    }

    /// save cookies manually
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn save_cookie<P: AsRef<Path>>(&self, path: P) {
        self.cookies.to_file(path);
    }

    /// save config manually
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn save_cred<P: AsRef<Path>>(&self, path: P) {
        self.cred.load().to_file(path);
    }
}

impl<G> Deref for Context<G> {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
