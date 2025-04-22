//! BUAA Elective Course API

mod auth;
mod opt;
mod utils;

pub use utils::*;

/// BUAA Elective Course API
///
/// Obtain a context view via [`Context.elective()`],
/// then call specific APIs through this grouping.
///
/// # Examples
/// ```
/// let ctx = Context::new();
/// let elective = ctx.elective();
/// elective.login().await.unwrap();
/// ```
///
/// Note: All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type ElectiveAPI = crate::Context<super::Elective>;
