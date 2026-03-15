//! # BUAA Classroom Live Broadcast (Spoc) API
//!
//! Including APIs related to classroom live broadcast and playback.
//!
//! In principle, it belongs to Spoc, but it mainly exists independently under `msa.buaa.edu.cn`.

mod core;
mod data;
mod opt;

pub use data::*;

/// # BUAA Classroom Live Broadcast (Spoc) API Group
///
/// Obtain a context view via [`Context.live()`],
/// then call specific APIs through this grouping.
///
/// ## Examples
///
/// ```
/// let ctx = Context::new();
/// let live = ctx.live();
/// live.login().await?;
/// ```
///
/// ## Note
///
/// All API groups share the same underlying context -
/// modifications will be instantly visible across all groups.
pub type LiveApi = crate::Context<super::Live>;
