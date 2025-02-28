//! BUAA Elective (本研选课) API
mod auth;
mod opt;
pub mod utils;

#[wrap_api::wrap_api(elective)]
struct ElectiveAPI;
