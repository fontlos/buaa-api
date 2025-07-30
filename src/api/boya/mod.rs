//! # BUAA Boya Course API

mod auth;
mod opt;
mod query;
mod universal;
mod utils;

pub use utils::*;

/// BUAA Boya Course API
///
/// Obtain a context view via [`Context.boya()`],
/// then call specific APIs through this grouping.
///
/// # Examples
/// ```
/// let ctx = Context::new();
/// let boya = ctx.boya();
/// boya.login().await.unwrap();
/// ```
///
/// Note: All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type BoyaApi = crate::Context<super::Boya>;
