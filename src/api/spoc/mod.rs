//! BUAA Spoc API

mod auth;
pub mod get_schedule;
mod universal_request;

crate::wrap_api!(
    /// BUAA Spoc API Wrapper <br>
    /// Call `spoc()` on `Context` to get an instance of this struct and call corresponding API on this instance.
    SpocAPI,
    spoc
);
