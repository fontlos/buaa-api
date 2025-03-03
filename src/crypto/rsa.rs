use rsa::pkcs8::DecodePublicKey;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};

use base64::{Engine as _, engine::general_purpose};

pub fn rsa(data: &str) -> String {
    // 逆向得到的, 硬编码进 JS, 理论上应该永远不变
    let key = "-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDlHMQ3B5GsWnCe7Nlo1YiG/YmH
dlOiKOST5aRm4iaqYSvhvWmwcigoyWTM+8bv2+sf6nQBRDWTY4KmNV7DBk1eDnTI
Qo6ENA31k5/tYCLEXgjPbEjCK9spiyB62fCT6cqOhbamJB0lcDJRO6Vo1m3dy+fD
0jbxfDVBBNtyltIsDQIDAQAB
-----END PUBLIC KEY-----";
    let mut rng = rand::thread_rng();
    // 解析公钥
    let public_key = RsaPublicKey::from_public_key_pem(key).expect("Failed to parse public key");
    let enc_data = public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, data.as_bytes())
        .expect("failed to encrypt");
    // 将加密结果转换为 Base64 字符串
    general_purpose::STANDARD.encode(&enc_data)
}
