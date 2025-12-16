//! Self-implemented CookieStore
//!
//! According to the RFC 6265 specification (https://datatracker.ietf.org/doc/html/rfc6265), we made the following simplifications:
//! 1. Do not consider the Max-Age and Expires attributes; the persistence and expiration are handled by the CredentialStore.
//! 2. Do not consider the HttpOnly attribute, because we are not in a scripting environment.
//! 3. Do not consider the old-style Domain attribute like `.example.com`; only consider the standard Domain attribute: `example.com`.
//! 4. Do not consider the SameSite attribute, since we have no cross-site attack risk.
//! 5. Do not consider the special meaning of Cookie prefixes.
//! Our implementation only covers the following:
//! When storing:
//! 1. Record the original Set-Cookie string.
//! 2. Record the end position of the name=value part. We assume this field always exists and is valid.
//! 3. Record the Path, Domain, and Secure attributes if they exist.
//! When sending:
//! 1. Filter cookies by Path (if not None).
//! 2. Filter cookies by Domain (if not None; otherwise treat as HostOnly Cookie). We assume they span at most one subdomain level.
//! 3. Filter cookies by the Secure attribute.

// 根据 RFC 6265 规范 (https://datatracker.ietf.org/doc/html/rfc6265), 我们做出如下简化:
// 1. 不考虑 Max-Age 和 Expires 属性, 持久化储存有效期交由 CredentialStore 处理
// 2. 不考虑 HttpOnly 属性, 因为我们不是脚本环境
// 3. 不考虑旧版 Domain 属性: `.example.com`. 只考虑标准 Domain 属性: `example.com`
// 4. 不考虑 SameSite 属性, 毕竟我们没有跨域攻击的风险
// 5. 不考虑 Cookie 前缀特殊含义
// 我们的实现只针对以下内容:
// 储存时:
// 1. 记录原始 Set-Cookie 字符串
// 2. 记录 name=value 部分的结束位置. 我们假定这一字段一定存在. 并且合法
// 3. 记录 Path, Domain, Secure 属性, 如果存在
// 发送时:
// 1. 通过 Path 过滤 Cookie (如果不为 None)
// 2. 通过 Domain 过滤 Cookie (如果不为 None, 否则视为 HostOnly Cookie). 并且我们假定最多只跨一级域名
// 3. 通过 Secure 属性过滤 Cookie

use reqwest::Url;
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::ops::Deref;
use std::path::Path;

use crate::cell::AtomicCell;
use crate::error::{Error, Result};

// 简易 Cookie 结构, 我们不关心额外信息, 能存能发即可
/// Simple Cookie
#[derive(Debug, Serialize, Deserialize)]
pub struct Cookie {
    raw: String,
    // 直接记录 name=value 部分的结束位置
    item: usize,
    // 考虑到极少情况下会有, 多数情况是 None, 直接使用 String 避免索引处理问题
    path: Option<String>,
    domain: Option<String>,
    secure: bool,
}

impl Cookie {
    /// Parse from `Set-Cookie` raw string
    pub fn parse(raw: &str) -> Cookie {
        let item_idx = raw.find(';').unwrap_or(raw.len());

        let mut path = None;
        let mut domain = None;
        let mut secure = false;

        for part in raw.split(';') {
            if let Some((k, v)) = part.trim().split_once('=') {
                match k.to_lowercase().as_str() {
                    // 小于等于 1 时为 '/' 或 没有
                    "path" if v.len() > 1 => {
                        path = Some(v.trim().to_string());
                    }
                    "domain" => {
                        domain = Some(v.trim().to_string());
                    }
                    _ => {}
                }
            } else {
                match part.trim().to_lowercase().as_str() {
                    "secure" => secure = true,
                    _ => {}
                }
            }
        }

        Cookie {
            raw: raw.to_string(),
            item: item_idx,
            path,
            domain,
            secure,
        }
    }

    /// Get raw cookie string
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Get cookie item (name=value)
    pub fn item(&self) -> &str {
        &self.raw[0..self.item]
    }

    /// Get cookie name
    pub fn name(&self) -> Option<&str> {
        self.item().split_once('=').map(|(name, _)| name)
    }

    /// Get cookie value
    pub fn value(&self) -> Option<&str> {
        self.item().split_once('=').map(|(_, value)| value)
    }

    fn path(&self) -> &str {
        self.path.as_deref().unwrap_or("/")
    }

    fn domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }

    // 匹配路径与安全属性, 至于 host 已经在 get 方法中处理过了
    fn matches_url(&self, url: &Url) -> bool {
        url.path().starts_with(self.path()) && (!self.secure || url.scheme() == "https")
    }
}

type NameMap = HashMap<String, Cookie>;
type DomainMap = HashMap<String, NameMap>;

/// Simple Cookie Store
#[derive(Debug, Serialize, Deserialize)]
pub struct CookieStore {
    store: DomainMap,
}

impl Default for CookieStore {
    fn default() -> Self {
        Self::new(DomainMap::new())
    }
}

impl CookieStore {
    /// Create a new empty cookie store
    fn new(store: DomainMap) -> Self {
        Self { store }
    }

    /// Load cookie store from file, if file not exist or invalid, return default store
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|e| Error::io("Failed to open cookies.json").with_source(e))?;
        let store: DomainMap = serde_json::from_reader(file)
            .map_err(|e| Error::io("Failed to read cookies.json").with_source(e))?;
        Ok(Self::new(store))
    }

    /// Save cookie store to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| Error::io("Failed to open cookies.json").with_source(e))?;
        serde_json::to_writer(&mut file, &self.store)
            .map_err(|e| Error::io("Failed to write cookies.json").with_source(e))?;
        Ok(())
    }

    /// Get cookies map by a domain
    pub fn get_map(&self, domain: &str) -> Option<&NameMap> {
        self.store.get(domain)
    }

    /// Get mutable cookies map by a domain
    pub fn get_mut_map(&mut self, domain: &str) -> Option<&mut NameMap> {
        self.store.get_mut(domain)
    }

    /// Get a cookie
    pub fn get(&self, domain: &str, name: &str) -> Option<&Cookie> {
        self.store
            .get(domain)
            .and_then(|name_map| name_map.get(name))
    }

    /// Insert a cookie
    pub fn insert(&mut self, domain: &str, cookie: Cookie) -> Option<Cookie> {
        let name = cookie.name()?;

        let name_map = self
            .store
            .entry(domain.to_string())
            .or_insert_with(HashMap::new);
        name_map.insert(name.to_string(), cookie)
    }

    /// Remove a cookie
    pub fn remove(&mut self, domain: &str, name: &str) -> Option<Cookie> {
        self.store
            .get_mut(domain)
            .and_then(|name_map| name_map.remove(name))
    }

    /// Store response cookies
    pub fn store_response_cookies<I>(&mut self, cookies: I, url: &Url)
    where
        I: Iterator<Item = Cookie>,
    {
        for cookie in cookies {
            if let Some(name) = cookie.name() {
                if name.is_empty() {
                    log::debug!("Skipping cookie with empty name: {}", cookie.raw());
                    continue;
                }
                if let Some(domain) = cookie.domain().or(url.host_str()) {
                    let name_map = self
                        .store
                        .entry(domain.to_string())
                        .or_insert_with(HashMap::new);
                    name_map.insert(name.to_string(), cookie);
                }
            } else {
                log::debug!("Skipping bad cookie (no 'name=value'): {}", cookie.raw());
            }
        }
    }

    /// Get request cookies iterator
    pub fn get_request_values(&self, url: &Url) -> impl Iterator<Item = &str> {
        // 如果解包 host 不合法, 使用空字符串也能返回空迭代器
        let host = url.host_str().unwrap_or("");

        // HostOnly Cookies 迭代器
        let exact_iter = self
            .store
            .get(host)
            .into_iter()
            .flat_map(|name_map| name_map.values())
            .filter(move |cookie| cookie.matches_url(url))
            .map(|cookie| cookie.item());

        // Suffix Domain Cookies 迭代器
        // 看看有没有需要带上的父域名 cookie
        // 不过考虑到实际应用我们只处理一级父域名即可
        let parent_iter = host
            .find('.')
            .and_then(|dot_pos| {
                let parent = &host[dot_pos + 1..];
                self.store.get(parent)
            })
            .into_iter()
            .flat_map(|name_map| name_map.values())
            // 如果确实设置了 domain 则尝试匹配
            .filter(move |cookie| cookie.domain().is_some() && cookie.matches_url(url))
            .map(|cookie| cookie.item());

        // 合并两个迭代器
        exact_iter.chain(parent_iter)
    }
}

/// Atomic cookie store
pub(crate) struct AtomicCookieStore(AtomicCell<CookieStore>);

impl Default for AtomicCookieStore {
    fn default() -> Self {
        AtomicCookieStore::new(CookieStore::default())
    }
}

impl Deref for AtomicCookieStore {
    type Target = AtomicCell<CookieStore>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AtomicCookieStore {
    /// Create a new atomic cookie store
    pub fn new(store: CookieStore) -> Self {
        AtomicCookieStore(AtomicCell::new(store))
    }
}

impl reqwest::cookie::CookieStore for AtomicCookieStore {
    fn set_cookies(
        &self,
        cookie_headers: &mut dyn Iterator<Item = &HeaderValue>,
        url: &reqwest::Url,
    ) {
        let cookies = cookie_headers
            .filter_map(|val| std::str::from_utf8(val.as_bytes()).map(Cookie::parse).ok());
        self.update(|store| {
            store.store_response_cookies(cookies, url);
        });
    }

    fn cookies(&self, url: &reqwest::Url) -> Option<HeaderValue> {
        let s = self
            .load()
            .get_request_values(url)
            .collect::<Vec<_>>()
            .join("; ");

        if s.is_empty() {
            return None;
        }

        HeaderValue::from_str(&s).ok()
    }
}
