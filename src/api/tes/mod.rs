//! # BUAA Teacher Evaluation System API

mod auth;
mod opt;
mod utils;

pub use utils::*;

/// BUAA Teacher Evaluation System API Group
///
/// Obtain a context view via [`Context.tes()`],
/// then call specific APIs through this grouping.
///
/// # Examples
/// ```
/// let ctx = Context::new();
/// let tes = ctx.tes();
/// tes.login().await.unwrap();
/// ```
///
/// Note: All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type TesApi = crate::Context<super::Tes>;
