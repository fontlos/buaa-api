//! BUAA Teacher Evaluation System (期末评教) API

mod opt;
mod utils;

pub use utils::*;

pub type EvaluationAPI = crate::Context<super::Evaluation>;
