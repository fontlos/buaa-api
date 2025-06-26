//! Self-implemented MD5, use to make dependencies minimize, and the performance gap is negligible for the small amount of data we pass on

#[cfg(not(feature = "crypto"))]
mod light {
    pub struct MD5 {
        state: [u32; 4],
        count: [u64; 2],
        buffer: [u8; 64],
    }

    impl MD5 {
        pub fn new() -> Self {
            MD5 {
                state: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476],
                count: [0, 0],
                buffer: [0; 64],
            }
        }

        pub fn update(&mut self, input: &[u8]) {
            let mut index = ((self.count[0] >> 3) & 0x3F) as usize;
            let len = input.len();

            self.count[0] += (len as u64) << 3;
            if self.count[0] < (len as u64) << 3 {
                self.count[1] += 1;
            }
            self.count[1] += (len as u64) >> 61;

            let mut i = 0;

            while i < len {
                self.buffer[index] = input[i];
                index += 1;
                i += 1;

                if index == 64 {
                    self.transform();
                    index = 0;
                }
            }
        }

        pub fn finalize(mut self) -> [u8; 16] {
            let mut bits = [0u8; 8];
            bits.copy_from_slice(&(self.count[0].to_le_bytes()));

            // 添加一个 1 bit 和七个 0 bits (0x80)
            self.update(&[0x80]);

            // 填充 0 直到长度 = 56 mod 64
            while ((self.count[0] >> 3) & 0x3F) != 56 {
                self.update(&[0]);
            }

            // 添加原始长度的低 64 位
            self.update(&bits);

            let mut digest = [0u8; 16];
            for (i, &word) in self.state.iter().enumerate() {
                digest[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
            }

            digest
        }

        fn transform(&mut self) {
            const S: [u32; 64] = [
                7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14,
                20, 5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11,
                16, 23, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
            ];

            const K: [u32; 64] = [
                0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
                0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
                0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
                0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
                0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
                0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
                0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
                0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
                0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
                0xeb86d391,
            ];

            let mut m = [0u32; 16];
            for (i, item) in m.iter_mut().enumerate() {
                *item = u32::from_le_bytes([
                    self.buffer[i * 4],
                    self.buffer[i * 4 + 1],
                    self.buffer[i * 4 + 2],
                    self.buffer[i * 4 + 3],
                ]);
            }

            let mut a = self.state[0];
            let mut b = self.state[1];
            let mut c = self.state[2];
            let mut d = self.state[3];

            for i in 0..64 {
                let (f, g) = match i {
                    0..=15 => ((b & c) | ((!b) & d), i),
                    16..=31 => ((d & b) | ((!d) & c), (5 * i + 1) % 16),
                    32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                    48..=63 => (c ^ (b | (!d)), (7 * i) % 16),
                    _ => unreachable!(),
                };

                let temp = d;
                d = c;
                c = b;
                b = b.wrapping_add(
                    (a.wrapping_add(f).wrapping_add(K[i]).wrapping_add(m[g])).rotate_left(S[i]),
                );
                a = temp;
            }

            self.state[0] = self.state[0].wrapping_add(a);
            self.state[1] = self.state[1].wrapping_add(b);
            self.state[2] = self.state[2].wrapping_add(c);
            self.state[3] = self.state[3].wrapping_add(d);
        }
    }

    pub struct HMACMD5 {
        key_block: [u8; 64],
    }

    impl HMACMD5 {
        pub fn new(key: &[u8]) -> Self {
            let mut key_block = [0u8; 64];

            // 如果密钥比块大小长，先哈希它，然后补零到块大小
            if key.len() > 64 {
                let mut hash = MD5::new();
                hash.update(key);
                let hash = hash.finalize();
                key_block[..16].copy_from_slice(&hash);
            } else {
                key_block[..key.len()].copy_from_slice(key);
            }

            HMACMD5 { key_block }
        }

        pub fn compute(&self, message: &[u8]) -> [u8; 16] {
            let mut k_ipad = [0x36u8; 64];
            let mut k_opad = [0x5cu8; 64];

            // 创建 ipad 和 opad 密钥
            for i in 0..64 {
                k_ipad[i] ^= self.key_block[i];
                k_opad[i] ^= self.key_block[i];
            }

            // 计算内部哈希 (k_ipad || message)
            let mut inner_msg = Vec::with_capacity(64 + message.len());
            inner_msg.extend_from_slice(&k_ipad);
            inner_msg.extend_from_slice(message);
            let mut inner_hash = MD5::new();
            inner_hash.update(&inner_msg);
            let inner_hash = inner_hash.finalize();

            // 计算外部哈希 (k_opad || inner_hash)
            let mut outer_msg = Vec::with_capacity(64 + 16);
            outer_msg.extend_from_slice(&k_opad);
            outer_msg.extend_from_slice(&inner_hash);
            let mut hash = MD5::new();
            hash.update(&outer_msg);
            hash.finalize()
        }
    }
}

#[cfg(not(feature = "crypto"))]
#[allow(dead_code)]
pub fn md5(data: &[u8]) -> String {
    let mut hasher = light::MD5::new();
    hasher.update(data);
    let result = hasher.finalize();
    crate::crypto::bytes2hex(&result)
}

#[cfg(not(feature = "crypto"))]
pub fn md5_hmac(data: &[u8], key: &[u8]) -> String {
    let hmac = light::HMACMD5::new(key);
    let digest = hmac.compute(data);
    crate::crypto::bytes2hex(&digest)
}

#[cfg(feature = "crypto")]
pub fn md5_hmac(data: &[u8], key: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use md5::Md5;
    let mut hmac = Hmac::<Md5>::new_from_slice(key).unwrap();
    hmac.update(data);
    let res = hmac.finalize().into_bytes();
    hex::encode(&res)
}
