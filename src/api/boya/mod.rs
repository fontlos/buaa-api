//! BUAA Boya (博雅选课) API

mod auth;
mod opt;
mod query;
mod universal;
mod utils;

pub use utils::*;

type BoyaAPI = crate::Context<super::Boya>;
