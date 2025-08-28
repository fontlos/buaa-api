pub mod aes;
pub mod des;
pub mod md5;
pub mod rsa;
pub mod sha1;
pub mod xencode;

mod test;

use base64::engine::{Engine, general_purpose};

/// Converts an array of bytes to a hexadecimal string
pub fn bytes2hex(bytes: &[u8]) -> String {
    #[cfg(not(feature = "crypto"))]
    {
        const HEX_LOWER: &[u8; 16] = b"0123456789abcdef";
        let mut hex_bytes = vec![0u8; bytes.len() * 2];
        for (i, &byte) in bytes.iter().enumerate() {
            hex_bytes[i * 2] = HEX_LOWER[(byte >> 4) as usize];
            hex_bytes[i * 2 + 1] = HEX_LOWER[(byte & 0xf) as usize];
        }
        unsafe { String::from_utf8_unchecked(hex_bytes) }
    }
    #[cfg(feature = "crypto")]
    {
        hex::encode(&bytes)
    }
}

pub fn encode_base64<T>(bytes: T) -> String
where
    T: AsRef<[u8]>,
{
    general_purpose::STANDARD.encode(bytes)
}

pub fn decode_base64<T>(s: T) -> Vec<u8>
where
    T: AsRef<[u8]>,
{
    general_purpose::STANDARD.decode(s).unwrap()
}