#![doc = include_str!("../Readme.md")]

mod api;
mod context;
mod crypto;
mod error;
pub mod utils;

pub use api::{
    boya::{
        query_course::{BoyaCampus, BoyaCourse, BoyaCourses, BoyaKind, BoyaTime},
        query_selected::{BoyaSelected, BoyaSelecteds},
        query_statistic::{BoyaAssessment, BoyaStatistic},
        BoyaAPI,
    },
    class::{ClassAPI, ClassCourse, ClassSchedule},
    spoc::{
        get_schedule::{SpocSchedule, SpocTimeRange, SpocWeek},
        SpocAPI,
    },
    user::UserCenterAPI,
    wifi::WiFiAPI,
};
pub use context::{Config, Context};
pub use error::{Error, Result};
