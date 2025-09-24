//! Credential manager

use serde::{Deserialize, Serialize};

use std::fs::OpenOptions;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::api::Location;
use crate::api::{Boya, Class, Cloud, Spoc, Srs, Sso};
use crate::error::{AuthError, Error, Result};
use crate::utils;

/// Store for credentials
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CredentialStore {
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    /// Token for Boya API
    pub boya_token: CredentialItem,
    /// Token for Class API
    pub class_token: CredentialItem,
    /// Token for Cloud API
    pub cloud_token: CredentialItem,
    /// Token for Spoc API
    pub spoc_token: CredentialItem,
    /// Token for Srs API
    pub srs_token: CredentialItem,
    /// Mark login expiration time of SSO
    pub sso: CredentialItem,
}

pub(crate) trait Token {
    const EXPIRATION: u64;
    fn field(store: &CredentialStore) -> &CredentialItem;
    fn mut_field(store: &mut CredentialStore) -> &mut CredentialItem;
    fn as_location() -> Location;
}

macro_rules! impl_token {
    ($type:ident, $field:ident, $expiration:expr) => {
        impl Token for $type {
            const EXPIRATION: u64 = $expiration;
            #[inline]
            fn field(store: &CredentialStore) -> &CredentialItem {
                &store.$field
            }
            #[inline]
            fn mut_field(store: &mut CredentialStore) -> &mut CredentialItem {
                &mut store.$field
            }
            #[inline]
            fn as_location() -> Location {
                Location::$type
            }
        }
    };
}

// 我们这里做保守估计防止 token 意外失效

// 测得 15 分钟以内有效, 这里用 10 分钟. 使用可刷新时效
impl_token!(Boya, boya_token, 600);
// 测得 7 天以内有效, 但 24 小时就够用了
impl_token!(Class, class_token, 86400);
// 测得 40 分钟以内有效, 这里用 30 分钟. 使用可刷新时效
impl_token!(Cloud, cloud_token, 1800);
// 测得 5 小时以内有效, 这里用 3 小时. 使用不可刷新时效
impl_token!(Spoc, spoc_token, 10800);
// 测得 25 分钟以内有效, 这里用 20 分钟. 使用不可刷新时效
impl_token!(Srs, srs_token, 1200);
// 测得 90 分钟以内有效. 使用可刷新时效
impl_token!(Sso, sso, 5400);

impl CredentialStore {
    /// Load credential store from file, if file not exist or invalid, return default store
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let file = match OpenOptions::new().read(true).open(path) {
            Ok(file) => file,
            Err(_) => return Self::default(),
        };
        serde_json::from_reader(file).unwrap_or_default()
    }

    /// Save credential store to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .unwrap();
        serde_json::to_writer(file, self).unwrap();
    }

    pub(crate) fn username(&self) -> Result<&str> {
        self.username
            .as_deref()
            .ok_or(Error::Auth(AuthError::NoUsername))
    }

    pub(crate) fn password(&self) -> Result<&str> {
        self.password
            .as_deref()
            .ok_or(Error::Auth(AuthError::NoPassword))
    }

    pub(crate) fn value<T: Token>(&self) -> Result<&str> {
        T::field(self)
            .value
            .as_deref()
            .ok_or(Error::Auth(AuthError::NoToken(T::as_location())))
    }

    pub(crate) fn is_expired<T: Token>(&self) -> bool {
        T::field(self).expiration.load(Ordering::Relaxed) < utils::get_time_secs()
    }

    // 原子类型可以直接在不可变引用上更新
    // 如果调用了 Update 就不需要这个方法了
    pub(crate) fn refresh<T: Token>(&self) {
        T::field(self)
            .expiration
            .store(utils::get_time_secs() + T::EXPIRATION, Ordering::Relaxed);
    }

    // Update 动作包含 Refresh
    pub(crate) fn update<T: Token>(&mut self, value: String) {
        let now = utils::get_time_secs();
        let item = T::mut_field(self);
        item.expiration
            .store(now + T::EXPIRATION, Ordering::Relaxed);
        item.value = Some(value);
        // 能用上 set 方法一定伴随着 Sso 的刷新
        // 而且 Sso 一定不会用上 set 方法
        // 所以这里可以放心的做一次 Sso 的刷新
        self.sso.expiration.store(now + 5400, Ordering::Relaxed);
    }
}

/// Credential item
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CredentialItem {
    value: Option<String>,
    expiration: AtomicU64,
}
