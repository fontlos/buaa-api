//! BUAA WiFi API

mod auth;

/// BUAA WiFi API Wrapper <br>
/// Call `wifi()` on `Context` to get an instance of this struct and call corresponding API on this instance.
#[wrap_api::wrap_api(wifi)]
struct WiFiAPI;
