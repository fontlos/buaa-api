//! # BUAA Cloud Disk API

mod core;
mod data;
mod opt;

pub use data::*;

/// # BUAA Cloud Disk API Group
///
/// Obtain a context view via [`Context.cloud()`],
/// then call specific APIs through this grouping.
///
/// ## Examples
///
/// ```
/// let ctx = Context::new();
/// let cloud = ctx.cloud();
/// cloud.login().await?;
/// ```
///
/// ## Note
///
/// All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type CloudApi = crate::Context<super::Cloud>;
