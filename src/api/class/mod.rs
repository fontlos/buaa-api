//! # BUAA Smart Classroom API
//!
//! It is used for class sign-in and class attendance inquiry

mod core;
mod data;
mod opt;

pub use data::*;

/// # BUAA Smart Classroom API
///
/// Obtain a context view via [`Context.class()`],
/// then call specific APIs through this grouping.
///
/// ## Examples
///
/// ```
/// let ctx = Context::new();
/// let class = ctx.class();
/// class.login().await?;
/// ```
///
/// ## Note
///
/// All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type ClassApi = crate::Context<super::Class>;
