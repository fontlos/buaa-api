#![doc = include_str!("../Readme.md")]

mod api;
mod crypto;
mod error;
mod session;
mod tests;
pub mod utils;

pub use api::{
    boya::{
        query_course::{BoyaCampus, BoyaCourse, BoyaKind, BoyaTime},
        query_selected::BoyaSelected,
        query_statistic::{BoyaAssessment, BoyaStatistic},
    },
    class::{ClassCourse, ClassSchedule},
    spoc::{SpocSchedule, SpocTimeRange, SpocWeek},
};
pub use error::SessionError;
pub use session::Session;
