mod app;
pub(crate) mod boya;
pub(crate) mod class;
pub(crate) mod spoc;
mod sso;
mod user;
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
mod wifi;
