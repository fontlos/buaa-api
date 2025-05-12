use cookie_store::{Cookie, CookieStore};
use reqwest::{
    Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use reqwest_cookie_store::CookieStoreMutex;

use std::fs::{self, File, OpenOptions};
use std::io::BufReader;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{api::SSO, cell::AtomicCell, store::cred::CredentialStore};

/// This is the core of this crate, it is used to store cookies and send requests <br>
pub struct Context<G = SSO> {
    pub(crate) client: Client,
    pub(crate) cookies: Arc<CookieStoreMutex>,
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
        let cookie_store = Arc::new(CookieStoreMutex::new(CookieStore::default()));
        let mut header = HeaderMap::new();
        header.insert(HeaderName::from_bytes(b"User-Agent").unwrap(), HeaderValue::from_bytes(b"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0").unwrap());

        let client = Client::builder()
            .default_headers(header)
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();

        Context {
            client,
            cookies: cookie_store,
            cred: AtomicCell::new(CredentialStore::default()),
            _marker: PhantomData,
        }
    }

    pub fn set_cred(&self, cred: CredentialStore) {
        self.cred.store(cred);
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

    /// Load config from path, if the path is not exist or parse failed, it will return a default one
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn with_config<P: AsRef<Path>>(&self, path: P) {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();
        let config = if let Ok(config) = serde_json::from_reader(file) {
            config
        } else {
            CredentialStore::default()
        };

        self.cred.store(config);
    }

    /// Load cookies file to set Session cookies and set `cookie_path`, if the path is not exist, it will create a new file, but It won't be saved until you call `save` method
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn with_cookies<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let path = PathBuf::from(path.as_ref());
        let cookie_store = match File::open(&path) {
            Ok(f) => {
                CookieStore::load_all(BufReader::new(f), |s| serde_json::from_str::<Cookie<'_>>(s))
                    .unwrap()
            }
            Err(_) => CookieStore::default(),
        };

        let mut cookie_lock = match self.cookies.lock() {
            Ok(c) => c,
            Err(_) => return Err(crate::Error::LockError),
        };
        *cookie_lock = cookie_store;
        Ok(())
    }

    /// save cookies manually
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn save_cookie<P: AsRef<Path>>(&self, path: P) {
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open cookie file: {:?}", e);
                return;
            }
        };
        let store = self.cookies.lock().unwrap();
        if let Err(e) =
            store.save_incl_expired_and_nonpersistent(&mut file, |s| serde_json::to_string(s))
        {
            eprintln!("Failed to save cookie store: {}", e);
        }
    }

    /// save config manually
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn save_cred<P: AsRef<Path>>(&self, path: P) {
        let cred = self.cred.load();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .unwrap();
        serde_json::to_writer(file, cred).unwrap();
    }
}

impl<G> Deref for Context<G> {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
