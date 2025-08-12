#[cfg(test)]
mod tests {
    use crate::crypto::*;

    #[test]
    fn test_aes_ecb() {
        let encrypted = aes::aes_encrypt_ecb(b"HelloWorld", b"SenQBA8xn6CQGNJs");
        assert_eq!("Kn2AACfzhA8YPsPPH3SgdA==", encrypted);
        let decrypted = aes::aes_decrypt_ecb(encrypted.as_bytes(), b"SenQBA8xn6CQGNJs");
        assert_eq!("HelloWorld", decrypted);
    }

    #[test]
    fn test_aes_encrypt_cbc() {
        let encrypted =
            aes::aes_encrypt_cbc(b"HelloWorld", b"inco12345678ocni", b"ocni12345678inco");
        assert_eq!("Qb5wy8PdDSUs6EgTzMX6Gw==", encrypted);
    }

    #[test]
    fn test_des() {
        let encrypted = des::des_encrypt(b"HelloWorld", b"Jyd#351*");
        assert_eq!(&encrypted, "e8c2f09cbf46cb0a70f11196330b1657");
    }

    #[test]
    fn test_md5() {
        let data = std::fs::read("License").unwrap();
        let md5 = md5::md5(&data);
        assert_eq!(&md5, "2817feea7bcabab5909f75866950e0d3");
    }

    #[test]
    fn test_md5_hmac() {
        let hmac = md5::md5_hmac(b"HelloWorld", b"Key");
        assert_eq!(&hmac, "219e14bef981f117479a7695dacb10c7");
    }

    #[test]
    fn test_sha1() {
        let sha1 = sha1::sha1(b"HelloWorld");
        assert_eq!(&sha1, "db8ac1c259eb89d4a131b253bacfca5f319d54f2");
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
