//! Boya Course API

pub mod query_course;
pub mod query_selected;
pub mod query_statistic;
mod util;

use serde::Deserialize;

crate::wrap_api!(BoyaAPI, boya);

#[derive(Deserialize)]
struct BoyaStatus {
    status: String,
    errmsg: String,
}
