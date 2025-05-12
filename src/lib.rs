#![doc = include_str!("../Readme.md")]

pub mod api;
mod cell;
mod consts;
mod context;
mod crypto;
mod error;
mod store;
pub mod utils;

pub use context::{Context, CredentialStore};
pub use error::{Error, Result};
