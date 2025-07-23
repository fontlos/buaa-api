//! # BUAA Cloud Disk API

mod auth;
// mod download;
mod opt;
mod universal;
mod utils;

/// BUAA Cloud Disk API Group
///
/// Obtain a context view via [`Context.cloud()`],
/// then call specific APIs through this grouping.
///
/// # Examples
/// ```
/// let ctx = Context::new();
/// let cloud = ctx.cloud();
/// cloud.login().await.unwrap();
/// ```
///
/// Note: All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type CloudAPI = crate::Context<super::Cloud>;
