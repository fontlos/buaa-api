//! BUAA Boya API

mod auth;
mod opt_course;
pub mod query_course;
pub mod query_selected;
pub mod query_statistic;
mod universal_request;

crate::wrap_api!(
    /// BUAA Boya API Wrapper <br>
    /// Call `boya()` on `Context` to get an instance of this struct and call corresponding API on this instance.
    BoyaAPI,
    boya
);
