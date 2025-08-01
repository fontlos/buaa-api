//! # BUAA Academic Affairs System API

/// # BUAA Academic Affairs System API Group
///
/// Obtain a context view via [`Context.aas()`],
/// then call specific APIs through this grouping.
///
/// ## Examples
///
/// ```
/// let ctx = Context::new();
/// let aas = ctx.aas();
/// aas.login().await?;
/// ```
///
/// ## Note
///
/// All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type AasApi = crate::Context<super::Aas>;
