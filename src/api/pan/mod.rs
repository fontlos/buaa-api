//! BUAA Pan API

mod auth;
mod opt;

/// BUAA Pan API Group
///
/// Obtain a context view via [`Context.pan()`],
/// then call specific APIs through this grouping.
///
/// # Examples
/// ```
/// let ctx = Context::new();
/// let pan = ctx.pan();
/// pan.login().await.unwrap();
/// ```
///
/// Note: All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type PanAPI = crate::Context<super::Pan>;
