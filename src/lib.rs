#![doc = include_str!("../Readme.md")]

mod api;
mod context;
mod crypto;
mod error;
pub mod utils;

pub use api::{
    boya::BoyaAPI,
    class::ClassAPI,
    spoc::{
        get_schedule::SpocWeek,
        SpocAPI,
    },
    user::UserCenterAPI,
    wifi::WiFiAPI,
};
pub use context::{Config, Context};
pub use error::{Error, Result};
