//! # BUAA App API

mod core;
mod data;
mod opt;

pub use data::*;

/// # BUAA App API Group
///
/// Obtain a context view via [`Context.app()`],
/// then call specific APIs through this grouping.
///
/// ## Examples
///
/// ```
/// let ctx = Context::new();
/// let app = ctx.app();
/// app.login().await?;
/// ```
///
/// ## Note
///
/// All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type AppApi = crate::Context<super::App>;
