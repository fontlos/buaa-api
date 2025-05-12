#![doc = include_str!("../Readme.md")]

pub mod api;
mod cell;
mod consts;
mod context;
mod crypto;
mod error;
pub mod utils;

pub use context::{CredentialStore, Context};
pub use error::{Error, Result};
