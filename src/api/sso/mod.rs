//! # BUAA Single Sign On API

mod auth;

/// # BUAA SSO API
///
/// Obtain a context view via [`Context.sso()`],
/// then call specific APIs through this grouping.
///
/// ## Examples
///
/// ```
/// let ctx = Context::new();
/// let sso = ctx.sso();
/// sso.login().await?;
/// // In fact, you can call `login` on `Context` directly to do the same
/// // ctx.login().await?;
/// ```
///
/// ## Note
///
/// All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type SsoApi = crate::Context<super::Sso>;
