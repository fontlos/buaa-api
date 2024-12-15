#![doc = include_str!("../Readme.md")]

mod api;
mod context;
mod crypto;
mod error;
mod tests;
pub mod utils;

// #[macro_use]
// pub(crate) use utils::wrap_api::wrap_api;

pub use api::{
    boya::{
        query_course::{BoyaCampus, BoyaCourse, BoyaCourses, BoyaKind, BoyaTime},
        query_selected::{BoyaSelected, BoyaSelecteds},
        query_statistic::{BoyaAssessment, BoyaStatistic},
    },
    class::{ClassCourse, ClassSchedule},
    spoc::{SpocSchedule, SpocTimeRange, SpocWeek},
};
pub use context::{Config, Context};
pub use error::{Error, Result};
