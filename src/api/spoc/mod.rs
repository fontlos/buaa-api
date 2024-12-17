//! BUAA Spoc API

mod auth;
pub mod get_schedule;
mod universal_request;

/// BUAA Spoc API Wrapper <br>
/// Call `spoc()` on `Context` to get an instance of this struct and call corresponding API on this instance.
#[wrap_api::wrap_api(spoc)]
struct SpocAPI;
