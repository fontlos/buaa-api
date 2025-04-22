//! BUAA Pan (北航云盘) API

mod auth;
mod opt;

pub type PanAPI = crate::Context<super::Pan>;
