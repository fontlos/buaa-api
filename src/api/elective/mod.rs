//! BUAA Elective (本研选课) API

mod auth;
mod opt;
mod utils;

pub use utils::*;

#[wrap_api::wrap_api(elective)]
struct ElectiveAPI;
