//! BUAA WiFi API

mod auth;

/// BUAA WiFi API Group
///
/// Obtain a context view for WiFi operations via [`Context.wifi()`],
/// then call specific APIs through this grouping.
///
/// # Examples
/// ```
/// let ctx = Context::new();
/// let wifi = ctx.wifi();
/// wifi.login("BUAA-WiFi");
/// ```
///
/// Note: All API groups share the same underlying context - modifications
/// will be instantly visible across all groups.
pub type WiFiAPI = crate::Context<super::WiFi>;
