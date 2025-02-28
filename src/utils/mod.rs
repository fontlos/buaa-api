mod env;

#[cfg(test)]
pub use env::env;

mod parse;
pub(crate) use parse::{get_value_by_lable, get_values_by_lable};

mod time;
pub use time::*;

#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
mod wifi;
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
pub use wifi::*;

pub mod boya {
    pub use crate::api::boya::{
        query_course::{BoyaCampus, BoyaCapacity, BoyaCourse, BoyaKind, BoyaTime},
        query_selected::BoyaSelected,
        query_statistic::{BoyaAssessment, BoyaStatistic},
    };
}
pub mod class {
    pub use crate::api::class::{ClassCourse, ClassSchedule};
}
pub mod evaluation {
    pub use crate::api::evaluation::utils::{EvaluationAnswer, EvaluationForm, EvaluationListItem};
}
pub mod spoc {
    pub use crate::api::spoc::opt::{SpocSchedule, SpocTimeRange, SpocWeek};
}
