//! BUAA Spoc (智学北航) API

mod auth;
mod opt;
mod universal;
mod utils;

pub use utils::*;

pub type SpocAPI = crate::Context<super::Spoc>;
