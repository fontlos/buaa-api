#![doc = include_str!("../Readme.md")]
#![warn(missing_docs)]

//! # For more information, check:
//!
//! - [`AasApi`](./api/aas/type.AasApi.html)
//! - [`BoyaApi`](./api/boya/type.BoyaApi.html)
//! - [`ClassApi`](./api/class/type.ClassApi.html)
//! - [`CloudApi`](./api/cloud/type.CloudApi.html)
//! - [`SpocApi`](./api/spoc/type.SpocApi.html)
//! - [`SrsApi`](./api/srs/type.SrsApi.html)
//! - [`SsoApi`](./api/sso/type.SsoApi.html)
//! - [`TesApi`](./api/tes/type.TesApi.html)
//! - [`UserApi`](./api/user/type.UserApi.html)
//! - [`WifiApi`](./api/wifi/type.WifiApi.html)

pub mod api;
mod cell;
mod context;
mod crypto;
pub mod error;
mod request;
pub mod store;
mod utils;

pub use context::Context;
pub use error::{Error, Result};
