//! # BUAA WiFi API

mod auth;

/// BUAA WiFi API Group
///
/// Obtain a context view via [`Context.wifi()`],
/// then call specific APIs through this grouping.
///
/// # Examples
/// ```
/// let ctx = Context::new();
/// let wifi = ctx.wifi();
/// wifi.login().await.unwrap();
/// ```
///
/// Note: All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type WifiAPI = crate::Context<super::Wifi>;
