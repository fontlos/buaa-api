#[cfg(test)]
mod tests {
    use crate::crypto::*;

    #[test]
    fn test_aes_ecb() {
        let cipher = aes::Aes128::new(b"SenQBA8xn6CQGNJs");
        let encrypted = cipher.encrypt_ecb(b"HelloWorld");
        let base64 = encode_base64(&encrypted);
        assert_eq!("Kn2AACfzhA8YPsPPH3SgdA==", base64);

        let encrypted = decode_base64(&base64);
        let decrypted = cipher.decrypt_ecb(&encrypted);
        assert_eq!(b"HelloWorld", decrypted.as_slice());
    }

    #[test]
    fn test_aes_encrypt_cbc() {
        let cipher = aes::Aes128::new(b"inco12345678ocni");
        let encrypted = cipher.encrypt_cbc(b"HelloWorld", b"ocni12345678inco");
        let base64 = encode_base64(&encrypted);
        assert_eq!("Qb5wy8PdDSUs6EgTzMX6Gw==", base64);
    }

    #[test]
    fn test_des() {
        let cipher = des::Des::new(b"Jyd#351*");
        let encrypted = cipher.encrypt_ecb(b"HelloWorld");
        let hex = bytes2hex(&encrypted);
        assert_eq!(&hex, "e8c2f09cbf46cb0a70f11196330b1657");
    }

    #[test]
    fn test_md5() {
        let data = std::fs::read("License").expect("Read License");
        let md5 = md5::Md5::digest(&data);
        let hex = bytes2hex(&md5);
        assert_eq!(&hex, "2817feea7bcabab5909f75866950e0d3");
    }

    #[test]
    fn test_md5_hmac() {
        let cipher = md5::HmacMd5::new(b"Key");
        let hmac = cipher.compute(b"HelloWorld");
        let hex = bytes2hex(&hmac);
        assert_eq!(&hex, "219e14bef981f117479a7695dacb10c7");
    }

    #[test]
    fn test_sha1() {
        let sha1 = sha1::Sha1::digest(b"HelloWorld");
        let hex = bytes2hex(&sha1);
        assert_eq!(&hex, "db8ac1c259eb89d4a131b253bacfca5f319d54f2");
    }

    #[test]
    fn test_xencoder() {
        let res = xencode::x_encode(
            b"HelloWorld",
            b"8e4e83f094924913acc6a9d5149015aafc898bd38ba8f45be6bd0f9edd450403",
        );
        assert_eq!(&res, "{SRBX1}9GAfJJT7wdSzFKeNohuv6+==");
    }
}
