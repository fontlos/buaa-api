#[cfg(not(feature = "crypto"))]
mod light {
    //! Self-implemented SHA1, use to make dependencies minimize, and the performance gap is negligible for the small amount of data we pass on
    use crate::utils::bytes_to_hex_fast;
    struct SHA1 {
        state: [u32; 5],
        count: [u64; 2],
        buffer: [u8; 64],
    }

    impl SHA1 {
        fn new() -> Self {
            SHA1 {
                state: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
                count: [0, 0],
                buffer: [0; 64],
            }
        }

        fn update(&mut self, input: &[u8]) {
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

        fn finalize(mut self) -> [u8; 20] {
            let mut bits = [0u8; 8];
            bits.copy_from_slice(&(self.count[0].to_be_bytes()));

            let pad_len = if ((self.count[0] >> 3) & 0x3F) < 56 {
                56 - ((self.count[0] >> 3) & 0x3F)
            } else {
                120 - ((self.count[0] >> 3) & 0x3F)
            } as usize;

            self.update(&[0x80]);
            self.update(&vec![0; pad_len - 1]);
            self.update(&bits);

            let mut digest = [0u8; 20];
            for (i, &word) in self.state.iter().enumerate() {
                digest[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
            }

            digest
        }

        fn transform(&mut self) {
            let mut w = [0u32; 80];

            for i in 0..16 {
                w[i] = u32::from_be_bytes([
                    self.buffer[i * 4],
                    self.buffer[i * 4 + 1],
                    self.buffer[i * 4 + 2],
                    self.buffer[i * 4 + 3],
                ]);
            }

            for i in 16..80 {
                w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
            }

            let mut a = self.state[0];
            let mut b = self.state[1];
            let mut c = self.state[2];
            let mut d = self.state[3];
            let mut e = self.state[4];

            for i in 0..80 {
                let (f, k) = match i {
                    0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                    20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                    40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                    60..=79 => (b ^ c ^ d, 0xCA62C1D6),
                    _ => unreachable!(),
                };

                let temp = a
                    .rotate_left(5)
                    .wrapping_add(f)
                    .wrapping_add(e)
                    .wrapping_add(k)
                    .wrapping_add(w[i]);

                e = d;
                d = c;
                c = b.rotate_left(30);
                b = a;
                a = temp;
            }

            self.state[0] = self.state[0].wrapping_add(a);
            self.state[1] = self.state[1].wrapping_add(b);
            self.state[2] = self.state[2].wrapping_add(c);
            self.state[3] = self.state[3].wrapping_add(d);
            self.state[4] = self.state[4].wrapping_add(e);
        }
    }

    pub fn sha1(data: &[u8]) -> String {
        let mut hasher = SHA1::new();
        hasher.update(data);
        let result = hasher.finalize();
        bytes_to_hex_fast(&result)
    }
}

#[cfg(not(feature = "crypto"))]
pub use light::*;

#[cfg(feature = "crypto")]
mod fast {
    pub fn sha1(data: &str) -> String {
        use sha1::{Digest, Sha1};
        let hasher = Sha1::digest(data.as_bytes());
        hex::encode(&hasher)
    }
}

#[cfg(feature = "crypto")]
pub use fast::*;
