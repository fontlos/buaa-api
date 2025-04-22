//! BUAA Elective Course API

mod auth;
mod opt;
mod utils;

pub use utils::*;

pub type ElectiveAPI = crate::Context<super::Elective>;
