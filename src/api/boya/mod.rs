//! BUAA Boya API

mod auth;
mod opt_course;
mod query_course;
mod query_selected;
mod query_statistic;
mod universal_request;
mod utils;

pub use utils::*;

/// BUAA Boya API Wrapper <br>
/// Call `boya()` on `Context` to get an instance of this struct and call corresponding API on this instance.
#[wrap_api::wrap_api(boya, vpn)]
struct BoyaAPI;
