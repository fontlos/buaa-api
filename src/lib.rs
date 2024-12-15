#![doc = include_str!("../Readme.md")]

mod api;
mod context;
mod crypto;
mod error;
mod tests;
pub mod utils;


pub use api::{
    boya::{
        BoyaAPI,
        query_course::{BoyaCampus, BoyaCourse, BoyaCourses, BoyaKind, BoyaTime},
        query_selected::{BoyaSelected, BoyaSelecteds},
        query_statistic::{BoyaAssessment, BoyaStatistic}
    },
    class::{ClassAPI, ClassCourse, ClassSchedule},
    spoc::{SpocAPI, SpocSchedule, SpocTimeRange, SpocWeek},
    user::UserCenterAPI,
    wifi::WiFiAPI
};
pub use context::{Config, Context};
pub use error::{Error, Result};
