//! BUAA Elective (本研选课) API

mod auth;
mod filter;
mod opt_course;
mod query_course;
mod query_selected;

#[wrap_api::wrap_api(elective)]
struct ElectiveAPI;
