//! # BUAA Spoc Platform API

mod auth;
mod opt;
mod universal;
mod utils;

pub use utils::*;

/// BUAA Spoc Platform API Group
///
/// Obtain a context view via [`Context.spoc()`],
/// then call specific APIs through this grouping.
///
/// # Examples
/// ```
/// let ctx = Context::new();
/// let spoc = ctx.spoc();
/// spoc.login().await.unwrap();
/// ```
///
/// Note: All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type SpocAPI = crate::Context<super::Spoc>;
