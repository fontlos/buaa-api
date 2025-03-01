//! BUAA Boya API

mod auth;
mod opt;
mod query;
mod universal;
mod utils;

pub use utils::*;

/// BUAA Boya API Wrapper <br>
/// Call `boya()` on `Context` to get an instance of this struct and call corresponding API on this instance.
#[wrap_api::wrap_api(boya, vpn)]
struct BoyaAPI;
