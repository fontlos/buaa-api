use cookie_store::{Cookie, CookieStore};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};
use reqwest_cookie_store::CookieStoreMutex;

use std::fs::{self, File};
use std::io::BufReader;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// This is the core of this crate, it is used to store cookies and send requests <br>
/// The prefix of most API names is derived from the fourth-level domain name of the corresponding domain name
// pub struct Context {
//     pub(crate) shared: Arc<SharedResources>,
// }

// impl Context {
//     /// Create a new context
//     pub fn new() -> Self {
//         Context {
//             shared: Arc::new(SharedResources::new()),
//         }
//     }

//     pub fn with_cookies(&self, path: &str) {
//         self.shared.with_cookies(path);
//     }

//     pub fn save(&self) {
//         self.shared.save();
//     }
// }

// impl std::ops::Deref for Context {
//     type Target = reqwest::Client;

//     fn deref(&self) -> &Self::Target {
//         &self.shared.client
//     }
// }

#[derive(Debug, Clone)]
pub struct Context {
    pub(crate) client: Client,
    pub(crate) cookies: Arc<CookieStoreMutex>,
    pub config: Arc<RwLock<Config>>,
}

#[derive(Debug)]
pub struct Config {
    pub cookie_path: Option<PathBuf>,
    pub vpn: bool,
    /// Token for Boya API
    pub boya_token: Option<String>,
    /// User ID for SmartClass API
    pub class_token: Option<String>,
    /// User ID for Spoc API
    pub spoc_token: Option<String>,
}

impl Context {
    /// Create a new session in memory, if you call `save` method, it will save cookies to `cookies.json` defaultly
    /// ```rust
    /// use buaa::Session;
    ///
    /// fn main() {
    ///     let mut session = Session::new();
    ///     // if you call `save` method, it will save cookies to `cookies.json` defaultly
    ///     // session.save();
    ///     // if you need load cookies from file, you can use `with_cookies` method
    ///     // session.with_cookies("path_to_cookies.json");
    ///     // and then you call `save` method will save cookies to the file you specified
    ///     // session.save();
    /// }
    /// ```
    pub fn new() -> Self {
        let cookie_store = Arc::new(CookieStoreMutex::new(CookieStore::default()));
        let mut header = HeaderMap::new();
        header.insert(HeaderName::from_bytes(b"User-Agent").unwrap(), HeaderValue::from_bytes(b"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0").unwrap());

        let client = Client::builder()
            .default_headers(header)
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();

        let config = Config {
            cookie_path: None,
            vpn: false,
            boya_token: None,
            class_token: None,
            spoc_token: None,
        };

        Context {
            client,
            cookies: cookie_store,
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Load cookies file to set Session cookies and set `cookie_path`, if the path is not exist, it will create a new file, but It won't be saved until you call `save` method
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn with_cookies(&self, path: &str) {
        let path = PathBuf::from(path);
        let cookie_store = match File::open(&path) {
            Ok(f) => {
                CookieStore::load_all(BufReader::new(f), |s| serde_json::from_str::<Cookie>(s))
                    .unwrap()
            }
            Err(_) => CookieStore::default(),
        };

        // TODO 记得处理锁失败的情况
        let mut cookie_lock = self.cookies.lock().unwrap();
        *cookie_lock = cookie_store;

        let mut config = self.config.write().unwrap();
        config.cookie_path = Some(path);
    }

    /// save cookies manually
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    pub fn save(&self) {
        // TODO 记得处理锁失败的情况
        let config = self.config.read().unwrap();
        let path = match &config.cookie_path {
            Some(p) => p.to_str().unwrap(),
            None => "cookies.json",
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
}

impl Deref for Context {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
