//! BUAA Spoc API

mod auth;
pub mod opt;
mod universal_request;

/// BUAA Spoc API Wrapper <br>
/// Call `spoc()` on `Context` to get an instance of this struct and call corresponding API on this instance.
#[wrap_api::wrap_api(spoc)]
struct SpocAPI;
