// mod app;
pub(crate) mod boya;
pub(crate) mod class;
mod office;
mod pan;
pub(crate) mod spoc;
mod sso;
mod user;
mod vpn;
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
mod wifi;
