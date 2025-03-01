mod opt;
mod utils;

pub use utils::*;

/// BUAA Teacher Evaluation System API Wrapper <br>
/// Call `evaluation()` on `Context` to get an instance of this struct and call corresponding API on this instance.
#[wrap_api::wrap_api(evaluation)]
struct EvaluationAPI;
