//! BUAA Smart Classroom (智慧教室) API
//!
//! It is used for class sign-in and class attendance inquiry

mod auth;
mod opt;
mod utils;

pub use utils::*;

type ClassAPI = crate::Context<super::Class>;
