//! # BUAA App API
//!
//! A mix of APIs located in `app.buaa.edu.cn`, including class schedules, etc.
//!
//! I don't know what these things are for, so it's **Not Recommended**

mod auth;
mod opt;

/// BUAA App API
///
/// Obtain a context view via [`Context.app()`],
/// then call specific APIs through this grouping.
///
/// # Examples
/// ```
/// let ctx = Context::new();
/// let app = ctx.app();
/// app.login().await.unwrap();
/// ```
///
/// Note: All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type AppApi = crate::Context<super::App>;
