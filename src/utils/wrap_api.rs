#[macro_export(local_inner_macros)]
macro_rules! wrap_api {
    ($api_wrap:ident, $api_fn:ident) => {
        #[allow(dead_code)]
        pub struct $api_wrap {
            client: reqwest::Client,
            cookies: std::sync::Arc<reqwest_cookie_store::CookieStoreMutex>,
            pub config: std::sync::Arc<std::sync::RwLock<crate::Config>>,
        }

        impl std::ops::Deref for $api_wrap {
            type Target = reqwest::Client;

            fn deref(&self) -> &Self::Target {
                &self.client
            }
        }

        impl crate::Context {
            pub fn $api_fn(&self) -> $api_wrap {
                $api_wrap {
                    client: self.client.clone(),
                    cookies: self.cookies.clone(),
                    config: self.config.clone(),
                }
            }
        }
    };
}
