mod env;

#[cfg(test)]
pub use env::env;

mod parse;
pub(crate) use parse::{get_value_by_lable, get_values_by_lable};

mod time;
pub use time::*;

#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
mod wifi;
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
pub use wifi::*;
