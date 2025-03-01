//! BUAA Smart Classroom (智慧教室) API
//!
//! It is used for class sign-in and class attendance inquiry

mod auth;
mod opt;
mod utils;

pub use utils::*;

/// BUAA Smart Classroom API Wrapper <br>
/// Call `class()` on `Context` to get an instance of this struct and call corresponding API on this instance.
#[wrap_api::wrap_api(class)]
struct ClassAPI;
