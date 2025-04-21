mod byte2hex;

pub mod aes;
pub mod des;

mod rsa;
pub use rsa::rsa;

#[cfg(feature = "crypto")]
/// High-performance implementation of the community, HMAC MD5 and SHA1
pub mod hash;

#[cfg(not(feature = "crypto"))]
mod light_hash;
#[cfg(not(feature = "crypto"))]
/// When the dependency is minimized, you can use your own implementation, and the performance gap is negligible for the small amount of data we pass the key
pub mod hash {
    pub use super::light_hash::*;
}

mod xencode;
pub use xencode::x_encode;

mod test;
