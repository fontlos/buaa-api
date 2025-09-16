use serde::{Deserialize, Serialize};

use std::fs::OpenOptions;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::api::Location;
use crate::api::{Boya, Class, Cloud, Spoc, Srs, Sso};
use crate::cell::AtomicCell;
use crate::error::{AuthError, Error, Result};
use crate::utils;

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
            fn as_location() -> Location {
                Location::$type
            }
        }
    };
}

// 经验证 15 分钟内过期, 我们这里用 10 分钟
impl_token!(Boya, boya_token, 600);
// 至少 7 天, 但即使更多对我们也用处不大了, 也许以后有时间我会测一测极限时间
impl_token!(Class, class_token, 604800);
// TODO: 我们先默认十分钟过期, 待测试
impl_token!(Cloud, cloud_token, 600);
// 至少 7 天, 但即使更多对我们也用处不大了, 也许以后有时间我会测一测极限时间
impl_token!(Spoc, spoc_token, 604800);
// TODO: 我们先默认十分钟过期, 待测试
impl_token!(Srs, srs_token, 600);
// 经验证 1.5 小时过期
impl_token!(Sso, sso, 5400);

impl CredentialStore {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let file = match OpenOptions::new().read(true).open(path) {
            Ok(file) => file,
            Err(_) => return Self::default(),
        };
        serde_json::from_reader(file).unwrap_or_default()
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .unwrap();
        serde_json::to_writer(file, self).unwrap();
    }

    const fn item(&mut self, loc: Location) -> (&mut CredentialItem, u64) {
        match loc {
            // 经验证 15 分钟内过期, 我们这里用 10 分钟
            Location::Boya => (&mut self.boya_token, 600),
            // 至少 7 天, 但即使更多对我们也用处不大了, 也许以后有时间我会测一测极限时间
            Location::Class => (&mut self.class_token, 604800),
            // TODO: 我们先默认十分钟过期, 待测试
            Location::Cloud => (&mut self.cloud_token, 600),
            // 至少 7 天, 但即使更多对我们也用处不大了, 也许以后有时间我会测一测极限时间
            Location::Spoc => (&mut self.spoc_token, 604800),
            // TODO: 我们先默认十分钟过期, 待测试
            Location::Srs => (&mut self.srs_token, 600),
            // 经验证 1.5 小时过期
            Location::Sso => (&mut self.sso, 5400),
            // 内部方法, 我们自己保证绝对不会出现其他分支
            _ => unreachable!(),
        }
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
}

impl AtomicCell<CredentialStore> {
    pub(crate) fn set(&self, loc: Location, value: String) {
        self.update(|store| {
            let now = utils::get_time_secs();
            let (item, expiration) = store.item(loc);
            item.expiration.store(now + expiration, Ordering::Relaxed);
            item.value = Some(value);
            // 能用上 set 方法一定伴随着 Sso 的刷新
            // 而且 Sso 一定不会用上 set 方法
            // 所以这里可以放心的做一次 Sso 的刷新
            store.sso.expiration.store(now + 5400, Ordering::Relaxed);
        });
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CredentialItem {
    value: Option<String>,
    expiration: AtomicU64,
}
