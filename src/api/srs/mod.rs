//! # BUAA Undergraduate & Graduate Student Course Registration System API

mod auth;
mod opt;
mod utils;

pub use utils::*;

/// BUAA Undergraduate & Graduate Student Course Registration System API
///
/// Obtain a context view via [`Context.srs()`],
/// then call specific APIs through this grouping.
///
/// # Examples
/// ```
/// let ctx = Context::new();
/// let srs = ctx.srs();
/// srs.login().await.unwrap();
/// ```
///
/// Note: All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type SrsAPI = crate::Context<super::Srs>;
