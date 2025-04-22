//! BUAA Teacher Evaluation System API

mod opt;
mod utils;

pub use utils::*;

/// BUAA Teacher Evaluation System API Group
///
/// Obtain a context view via [`Context.evaluation()`],
/// then call specific APIs through this grouping.
///
/// # Examples
/// ```
/// let ctx = Context::new();
/// let evaluation = ctx.evaluation();
/// evaluation.login().await.unwrap();
/// ```
///
/// Note: All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type EvaluationAPI = crate::Context<super::Evaluation>;
