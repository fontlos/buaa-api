//! # BUAA User Center API

mod auth;
mod opt;

/// # BUAA User Center API Group
///
/// Obtain a context view via [`Context.user()`],
/// then call specific APIs through this grouping.
///
/// ## Examples
///
/// ```
/// let ctx = Context::new();
/// let user = ctx.user();
/// user.login().await?;
/// ```
///
/// ## Note
///
/// All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type UserApi = crate::Context<super::User>;
