#![doc = include_str!("../Readme.md")]

mod api;
mod crypto;
mod error;
mod session;
mod tests;
pub mod utils;

pub use api::{
    boya::BoyaCourse,
    class::{ClassCourse, ClassSchedule},
    spoc::{SpocWeek, SpocSchedule},
};
pub use error::SessionError;
pub use session::Session;
