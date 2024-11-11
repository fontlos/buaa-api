use hmac::{Hmac, Mac};
use md5::Md5;
use sha1::{Sha1, Digest};

pub fn md5_hmac(data: &str, key: &str) -> String {
    let mut hmac = Hmac::<Md5>::new_from_slice(key.as_bytes()).unwrap();
    hmac.update(data.as_bytes());
    let res = hmac.finalize().into_bytes();
    hex::encode(&res)
}

pub fn sha1(data: &str) -> String {
    let hasher = Sha1::digest(data.as_bytes());
    hex::encode(&hasher)
}