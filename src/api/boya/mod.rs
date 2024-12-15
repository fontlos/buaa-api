//! BUAA Boya API

pub mod query_course;
pub mod query_selected;
pub mod query_statistic;
mod util;

use serde::Deserialize;

crate::wrap_api!(
    /// BUAA Boya API Wrapper <br>
    /// Call `boya()` on `Context` to get an instance of this struct and call corresponding API on this instance.
    BoyaAPI,
    boya
);

#[derive(Deserialize)]
struct BoyaStatus {
    status: String,
    errmsg: String,
}
