#![doc = include_str!("../Readme.md")]

mod api;
mod crypto;
mod error;
mod context;
mod tests;
pub mod utils;

pub use api::{
    boya::{
        query_course::{BoyaCampus, BoyaCourse, BoyaCourses, BoyaKind, BoyaTime},
        query_selected::{BoyaSelected, BoyaSelecteds},
        query_statistic::{BoyaAssessment, BoyaStatistic},
    },
    class::{ClassCourse, ClassSchedule},
    spoc::{SpocSchedule, SpocTimeRange, SpocWeek},
};
pub use error::{Error, Result};
pub use context::{SharedResources, Context};
