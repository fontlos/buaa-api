//! # BUAA Teaching Evaluation System API
//!
//! **Warning!**: Due to the poor design of the evaluation system server,
//! using this API may cause the evaluation button on the web page to become unclickable.
//! But don't worry, the evaluation data has been submitted correctly.
//! If you want to view the evaluation results on the web page,
//! you can remove the 'disabled' attribute of the button in the browser console,
//! and you'll be able to click it.
//! Or you might wait a little longer, and it may return to normal.

mod auth;
mod data;
mod opt;

pub use data::*;

/// # BUAA Teaching Evaluation System API Group
///
/// Obtain a context view via [`Context.tes()`],
/// then call specific APIs through this grouping.
///
/// ## Examples
///
/// ```
/// let ctx = Context::new();
/// let tes = ctx.tes();
/// tes.login().await?;
/// ```
///
/// ## Note
///
/// All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type TesApi = crate::Context<super::Tes>;
