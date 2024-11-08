#![doc = include_str!("../Readme.md")]

#[cfg(feature = "js")]
mod gw;
mod sso;
mod uc;
mod utils;

pub use sso::{Session, SessionError};
