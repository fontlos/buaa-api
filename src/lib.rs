#![doc = include_str!("../Readme.md")]

mod api;
mod context;
mod crypto;
mod error;
pub mod utils;

pub mod exports {
    pub mod boya {
        pub use crate::api::boya::{
            query_course::{BoyaCampus, BoyaCapacity, BoyaCourse, BoyaCourses, BoyaKind, BoyaTime},
            query_selected::{BoyaSelected, BoyaSelecteds},
            query_statistic::{BoyaAssessment, BoyaStatistic},
        };
    }
    pub mod class {
        pub use crate::api::class::{ClassCourse, ClassSchedule};
    }
    pub mod spoc {
        pub use crate::api::spoc::get_schedule::{SpocSchedule, SpocTimeRange, SpocWeek};
    }
}

pub use api::{boya::BoyaAPI, class::ClassAPI, spoc::SpocAPI, user::UserCenterAPI, wifi::WiFiAPI};
pub use context::{Config, Context};
pub use error::{Error, Result};
