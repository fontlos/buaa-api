pub mod boya;
pub mod class;
pub mod elective;
pub mod evaluation;
pub mod office;
pub mod pan;
pub mod spoc;
mod sso;
pub mod user;
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
pub mod wifi;
