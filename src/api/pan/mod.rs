//! BUAA Pan (北航云盘) API

mod auth;
mod opt;

/// BUAA Pan API Wrapper <br>
/// Call `pan()` on `Context` to get an instance of this struct and call corresponding API on this instance.
#[wrap_api::wrap_api(pan)]
struct PanAPI;
