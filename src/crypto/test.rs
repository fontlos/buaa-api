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
    fn test_crc() {
        let data = std::fs::read("License").expect("Read License");
        let crc32 = crc::Crc32::digest(&data);
        let hex = format!("{:08x}", crc32);
        assert_eq!(&hex, "6d3f72ad");
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

    // Rand

    #[test]
    fn test_u8_boundaries() {
        use crate::crypto::rand::{Rng, WyRng};
        let mut rng = WyRng::new();
        // 最小范围
        assert_eq!(rng.random_range(42u8..43), 42);
        assert_eq!(rng.random_range(0u8..=0), 0);
        assert_eq!(rng.random_range(255u8..=255), 255);
        // 最大范围, 注意完整范围 MIN..=MAX 不能用该方法, 会溢出
        for _ in 0..1000 {
            let val = rng.random_range(0u8..255);
            assert!(val < 255);
        }
        // 边界附近的窄范围
        for _ in 0..1000 {
            let val = rng.random_range(250u8..=255);
            assert!(val >= 250);
            let val = rng.random_range(0u8..5);
            assert!(val < 5);
        }
    }

    #[test]
    fn test_i32_boundaries() {
        use crate::crypto::rand::{Rng, WyRng};
        let mut rng = WyRng::new();
        // 最小范围
        assert_eq!(rng.random_range(42i32..43), 42);
        assert_eq!(rng.random_range(0i32..=0), 0);
        assert_eq!(rng.random_range(-1i32..=-1), -1);
        // 正数边界
        for _ in 0..1000 {
            let val = rng.random_range(i32::MAX - 10..=i32::MAX);
            assert!(val >= i32::MAX - 10 && val <= i32::MAX);
        }
        // 负数边界
        for _ in 0..1000 {
            let val = rng.random_range(i32::MIN..=i32::MIN + 10);
            assert!(val >= i32::MIN && val <= i32::MIN + 10);
        }
        // 跨零范围
        for _ in 0..1000 {
            let val = rng.random_range(-50i32..50);
            assert!(val >= -50 && val < 50);
        }
        // 整个范围
        for _ in 0..1000 {
            let val = rng.random_range(i32::MIN..i32::MAX);
            assert!(val >= i32::MIN && val < i32::MAX);
        }
    }

    #[test]
    fn test_int_boundaries() {
        use crate::crypto::rand::{Rng, WyRng};
        let mut rng = WyRng::new();
        // i8
        for _ in 0..1000 {
            let val = rng.random_range(i8::MIN..i8::MAX);
            assert!(val >= i8::MIN && val < i8::MAX);
            let val = rng.random_range(i8::MIN..=i8::MIN + 10);
            assert!(val >= i8::MIN && val <= i8::MIN + 10);
            let val = rng.random_range(i8::MAX - 10..=i8::MAX);
            assert!(val >= i8::MAX - 10 && val <= i8::MAX);
        }
        // u16
        for _ in 0..1000 {
            let val = rng.random_range(0u16..u16::MAX);
            assert!(val < u16::MAX);
            let val = rng.random_range(u16::MAX - 100..=u16::MAX);
            assert!(val >= u16::MAX - 100 && val <= u16::MAX);
        }
        // i64
        for _ in 0..1000 {
            let val = rng.random_range(i64::MIN..i64::MAX);
            assert!(val >= i64::MIN && val < i64::MAX);
            let val = rng.random_range(i64::MIN..=i64::MIN + 1000);
            assert!(val >= i64::MIN && val <= i64::MIN + 1000);
            let val = rng.random_range(i64::MAX - 1000..=i64::MAX);
            assert!(val >= i64::MAX - 1000 && val <= i64::MAX);
        }
        // usize/isize
        #[cfg(target_pointer_width = "64")]
        {
            for _ in 0..1000 {
                let val = rng.random_range(0usize..usize::MAX);
                assert!(val < usize::MAX);
                let val = rng.random_range(isize::MIN..isize::MAX);
                assert!(val >= isize::MIN && val < isize::MAX);
            }
        }
        #[cfg(target_pointer_width = "32")]
        {
            for _ in 0..1000 {
                let val = rng.random_range(0usize..usize::MAX);
                assert!(val < usize::MAX);
                let val = rng.random_range(isize::MIN..isize::MAX);
                assert!(val >= isize::MIN && val < isize::MAX);
            }
        }
    }

    #[test]
    fn test_f64_boundaries() {
        use crate::crypto::rand::{Rng, WyRng};
        let mut rng = WyRng::new();
        // 最小范围
        let val = rng.random_range(0.0f64..=0.0);
        assert_eq!(val, 0.0);
        // 小范围
        for _ in 0..1000 {
            let val = rng.random_range(0.0f64..1.0);
            assert!(val >= 0.0 && val < 1.0);
            let val = rng.random_range(-1.0f64..1.0);
            assert!(val >= -1.0 && val < 1.0);
            let val = rng.random_range(-1e-5f64..=1e-5);
            assert!(val >= -1e-5 && val <= 1e-5);
        }
        // 大范围
        for _ in 0..1000 {
            let val = rng.random_range(-1e6f64..1e6);
            assert!(val >= -1e6 && val < 1e6);
        }
    }

    #[test]
    fn test_distribution() {
        use crate::crypto::rand::{Rng, WyRng};
        let mut rng = WyRng::new();
        // u8 分布
        let mut u8_counts = [0u32; 10]; // 0-9
        for _ in 0..10000 {
            let val = rng.random_range(0u8..10);
            u8_counts[val as usize] += 1;
        }
        let expected = 1000;
        for (i, &count) in u8_counts.iter().enumerate() {
            let ratio = count as f64 / expected as f64;
            assert!(
                ratio > 0.8 && ratio < 1.2,
                "Value {} is unevenly distributed: expected ~{}, actual {}, proportion {:.2}",
                i,
                expected,
                count,
                ratio
            );
        }
        // i32 正负分布
        let mut neg_counts = 0;
        for _ in 0..10000 {
            let val = rng.random_range(-50i32..50);
            if val < 0 {
                neg_counts += 1;
            }
        }
        let neg_ratio = neg_counts as f64 / 10000.0;
        assert!(
            neg_ratio > 0.45 && neg_ratio < 0.55,
            "Negative and positive distribution is uneven: negative ratio {:.2}",
            neg_ratio
        );
        // f64 分布
        let mut buckets = [0u32; 10];
        for _ in 0..10000 {
            let val = rng.random_range(0.0f64..1.0);
            let bucket = (val * 10.0).floor() as usize;
            if bucket < 10 {
                buckets[bucket] += 1;
            }
        }
        for (i, &count) in buckets.iter().enumerate() {
            let ratio = count as f64 / 1000.0; // 每桶期望1000个
            assert!(
                ratio > 0.7 && ratio < 1.3,
                "f64 bucket {} uneven distribution: ratio {:.2}",
                i,
                ratio
            );
        }
    }
}
