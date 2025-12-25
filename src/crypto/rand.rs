//! Self-implemented Rng

use std::ops::{Range, RangeInclusive};

use crate::utils::get_time_nanos;

/// Random number generator core trait
pub trait RngCore {
    /// Generate a random byte
    fn next_byte(&mut self) -> u8;
    /// Generate a random u32
    fn next_u32(&mut self) -> u32;
    /// Generate a random u64
    fn next_u64(&mut self) -> u64;
    /// Fill bytes into the destination slice
    fn fill_bytes(&mut self, dest: &mut [u8]);
}

/// Random number generator trait
pub trait Rng: RngCore {
    /// Generate a random value from range
    ///
    /// **Warning!**: When range is (MIN..=MAX), it will overflow!
    /// And you should call `next_bytes`, `next_u32` or `next_u64` directly.
    fn random_range<T: SampleRange>(&mut self, range: T) -> T::Output
    where
        T::Output: PartialOrd + Copy,
        Self: Sized,
    {
        range.sample(self)
    }
}

/// WyRng: Very fast, suitable for non-cryptographic use
pub struct WyRng {
    state: u64,
}

impl WyRng {
    /// Create a new WyRng
    pub fn new() -> Self {
        let seed = get_time_nanos() as u64;
        Self {
            // 确保非零
            state: seed | 1,
        }
    }
}

impl RngCore for WyRng {
    fn next_byte(&mut self) -> u8 {
        (self.next_u64() >> 56) as u8
    }

    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let result = self.state;
        self.state = result.rotate_left(39) ^ result.rotate_left(22);
        result ^ (result >> 35)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut chunks = dest.chunks_exact_mut(8);
        for chunk in &mut chunks {
            let val = self.next_u64();
            chunk.copy_from_slice(&val.to_le_bytes());
        }

        let remainder = chunks.into_remainder();
        if !remainder.is_empty() {
            let val = self.next_u64();
            let bytes = val.to_le_bytes();
            remainder.copy_from_slice(&bytes[..remainder.len()]);
        }
    }
}

impl Rng for WyRng {}

/// Xoshiro256++Rng: Balances speed and statistical properties
pub struct Xoshiro256ppRng {
    state: [u64; 4],
}

impl Xoshiro256ppRng {
    /// Create a new Xoshiro256ppRng
    pub fn new() -> Self {
        let seed = get_time_nanos();
        let seed_bytes = seed.to_le_bytes();

        let mut state = [
            u64::from_le_bytes(seed_bytes[0..8].try_into().unwrap()),
            u64::from_le_bytes(seed_bytes[8..16].try_into().unwrap()),
            0,
            0,
        ];

        let mut mixer = state[0] ^ state[1];
        for i in 2..4 {
            mixer = mixer
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            state[i] = mixer;
        }

        // 确保非零
        if state.iter().all(|&x| x == 0) {
            state = [0xBAD5EEDu64; 4];
        }

        Self { state }
    }

    /// Create a new Xoshiro256ppRng from a seed
    pub fn from_seed(seed: [u64; 4]) -> Self {
        Self { state: seed }
    }
}

impl RngCore for Xoshiro256ppRng {
    fn next_byte(&mut self) -> u8 {
        (self.next_u64() >> 56) as u8
    }

    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    fn next_u64(&mut self) -> u64 {
        // 这是Xoshiro256++的关键改进：更好的输出函数
        let result = (self.state[0].wrapping_add(self.state[3]))
            .rotate_left(23)
            .wrapping_add(self.state[0]);

        // Xoshiro的跳跃函数
        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);

        result
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut chunks = dest.chunks_exact_mut(8);

        for chunk in &mut chunks {
            let val = self.next_u64();
            chunk.copy_from_slice(&val.to_le_bytes());
        }

        let remainder = chunks.into_remainder();
        if !remainder.is_empty() {
            let val = self.next_u64();
            let bytes = val.to_le_bytes();
            remainder.copy_from_slice(&bytes[..remainder.len()]);
        }
    }
}

impl Rng for Xoshiro256ppRng {}

/// ChaCha20Rng: Cryptographically secure
pub struct ChaCha20Rng {
    state: [u32; 16],
    buffer: [u8; 64],
    buffer_index: usize,
}

impl ChaCha20Rng {
    const BLOCK_SIZE: usize = 64;

    /// Create a new ChaCha20Rng
    pub fn new() -> Self {
        let seed = Self::generate_seed();
        Self::from_seed(seed)
    }

    /// Generate seed
    pub fn generate_seed() -> [u8; 32] {
        let mut seed = [0u8; 32];

        let time = get_time_nanos() as u64;
        let stack = &time as *const _ as u64;
        let phi = stack.wrapping_mul(0x9E3779B97F4A7C15);
        let mut mixer = time ^ stack ^ phi;

        for i in 0..32 {
            seed[i] = ((mixer >> ((i % 8) * 8)) as u8)
                .wrapping_add(i as u8)
                .rotate_left((i % 7 + 1) as u32);
            mixer = mixer
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
        }
        seed
    }

    /// Create a new ChaCha20Rng from a seed
    pub fn from_seed(seed: [u8; 32]) -> Self {
        // ChaCha的常量
        let mut state = [0u32; 16];

        // 设置魔数
        state[0] = 0x61707865;
        state[1] = 0x3320646e;
        state[2] = 0x79622d32;
        state[3] = 0x6b206574;

        // 设置密钥
        for i in 0..8 {
            let start = i * 4;
            state[4 + i] = u32::from_le_bytes([
                seed[start],
                seed[start + 1],
                seed[start + 2],
                seed[start + 3],
            ]);
        }

        state[12] = 0; // 计数器低32位
        state[13] = 0; // 计数器高32位
        state[14] = 0; // nonce低32位
        state[15] = 0; // nonce高32位

        Self {
            state,
            buffer: [0u8; 64],
            buffer_index: Self::BLOCK_SIZE,
        }
    }

    fn generate_block(&mut self) {
        let mut working_state = self.state;

        // ChaCha20: 20 轮 = 10 次双轮
        for _ in 0..10 {
            // 列轮
            Self::quarter_round(&mut working_state, 0, 4, 8, 12);
            Self::quarter_round(&mut working_state, 1, 5, 9, 13);
            Self::quarter_round(&mut working_state, 2, 6, 10, 14);
            Self::quarter_round(&mut working_state, 3, 7, 11, 15);

            // 对角线轮
            Self::quarter_round(&mut working_state, 0, 5, 10, 15);
            Self::quarter_round(&mut working_state, 1, 6, 11, 12);
            Self::quarter_round(&mut working_state, 2, 7, 8, 13);
            Self::quarter_round(&mut working_state, 3, 4, 9, 14);
        }

        // 加到初始状态
        for i in 0..16 {
            working_state[i] = working_state[i].wrapping_add(self.state[i]);
        }

        // 转换为字节
        for (i, &word) in working_state.iter().enumerate() {
            let bytes = word.to_le_bytes();
            let start = i * 4;
            self.buffer[start..start + 4].copy_from_slice(&bytes);
        }

        // 更新计数器
        self.state[12] = self.state[12].wrapping_add(1);
        if self.state[12] == 0 {
            self.state[13] = self.state[13].wrapping_add(1);
        }

        self.buffer_index = 0;
    }

    // 四分之一轮
    fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
        state[a] = state[a].wrapping_add(state[b]);
        state[d] ^= state[a];
        state[d] = state[d].rotate_left(16);

        state[c] = state[c].wrapping_add(state[d]);
        state[b] ^= state[c];
        state[b] = state[b].rotate_left(12);

        state[a] = state[a].wrapping_add(state[b]);
        state[d] ^= state[a];
        state[d] = state[d].rotate_left(8);

        state[c] = state[c].wrapping_add(state[d]);
        state[b] ^= state[c];
        state[b] = state[b].rotate_left(7);
    }
}

impl RngCore for ChaCha20Rng {
    fn next_byte(&mut self) -> u8 {
        if self.buffer_index >= Self::BLOCK_SIZE {
            self.generate_block();
        }

        let byte = self.buffer[self.buffer_index];
        self.buffer_index += 1;
        byte
    }

    fn next_u32(&mut self) -> u32 {
        if self.buffer_index + 4 <= Self::BLOCK_SIZE {
            let result = u32::from_le_bytes([
                self.buffer[self.buffer_index],
                self.buffer[self.buffer_index + 1],
                self.buffer[self.buffer_index + 2],
                self.buffer[self.buffer_index + 3],
            ]);
            self.buffer_index += 4;
            result
        } else {
            let mut bytes = [0u8; 4];
            self.fill_bytes(&mut bytes);
            u32::from_le_bytes(bytes)
        }
    }

    fn next_u64(&mut self) -> u64 {
        if self.buffer_index + 8 <= Self::BLOCK_SIZE {
            let result = u64::from_le_bytes([
                self.buffer[self.buffer_index],
                self.buffer[self.buffer_index + 1],
                self.buffer[self.buffer_index + 2],
                self.buffer[self.buffer_index + 3],
                self.buffer[self.buffer_index + 4],
                self.buffer[self.buffer_index + 5],
                self.buffer[self.buffer_index + 6],
                self.buffer[self.buffer_index + 7],
            ]);
            self.buffer_index += 8;
            result
        } else {
            let mut bytes = [0u8; 8];
            self.fill_bytes(&mut bytes);
            u64::from_le_bytes(bytes)
        }
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut offset = 0;

        while offset < dest.len() {
            if self.buffer_index >= Self::BLOCK_SIZE {
                self.generate_block();
            }

            let available = Self::BLOCK_SIZE - self.buffer_index;
            let needed = dest.len() - offset;
            let to_copy = available.min(needed);

            dest[offset..offset + to_copy]
                .copy_from_slice(&self.buffer[self.buffer_index..self.buffer_index + to_copy]);

            self.buffer_index += to_copy;
            offset += to_copy;
        }
    }
}

impl Rng for ChaCha20Rng {}

/// Trait for sampling from a range
pub trait SampleRange {
    /// The output type of the sampling
    type Output;

    /// Sample a random value from the range
    fn sample<R: RngCore>(&self, rng: &mut R) -> Self::Output;
}

// impl SampleRange for Range<i32> {
//     type Output = i32;

//     fn sample<R: RngCore>(&self, rng: &mut R) -> i32 {
//         let start = self.start;
//         let end = self.end;

//         if start == end {
//             return start;
//         }

//         let start = (start as u32) ^ (1u32 << 31);
//         let end = (end as u32) ^ (1u32 << 31);

//         let range = end - start;

//         // 注意这不是无偏的, 模运算的余数出现概率在模数不是 2 的幂时会有轻微偏差
//         let random_val = rng.next_u32() as u32 % range;
//         ((start + random_val)^(1u32 << 31)) as i32
//     }
// }

// impl SampleRange for RangeInclusive<i32> {
//     type Output = i32;

//     fn sample<R: RngCore>(&self, rng: &mut R) -> i32 {
//         let start = *self.start();
//         let end = *self.end();

//         if start == end {
//             return start;
//         }

//         let start = (start as u32) ^ (1u32 << 31);
//         let end = (end as u32) ^ (1u32 << 31);

//         let range = end - start;

//         debug_assert_ne!(range, u32::MAX, "RangeInclusive<MIN..=MAX> overflow!");

//         // 注意这不是无偏的, 模运算的余数出现概率在模数不是 2 的幂时会有轻微偏差
//         let random_val = rng.next_u32() as u32 % (range + 1);
//         ((start + random_val)^(1u32 << 31)) as i32
//     }
// }

macro_rules! impl_unsigned_sample_range {
    ($ty:ty, $rng_method:ident) => {
        impl SampleRange for std::ops::Range<$ty> {
            type Output = $ty;

            fn sample<R: RngCore>(&self, rng: &mut R) -> $ty {
                if self.start == self.end {
                    return self.start;
                }

                let range = self.end - self.start;
                // 把随机数截断到目标类型
                // 注意这不是无偏的, 模运算的余数出现概率在模数不是 2 的幂时会有轻微偏差
                let random_val = rng.$rng_method() as $ty % range;
                self.start + random_val
            }
        }

        impl SampleRange for std::ops::RangeInclusive<$ty> {
            type Output = $ty;

            fn sample<R: RngCore>(&self, rng: &mut R) -> $ty {
                let start = *self.start();
                let end = *self.end();

                if start == end {
                    return start;
                }

                let range = end - start;
                // 把随机数截断到目标类型
                // 注意这不是无偏的, 模运算的余数出现概率在模数不是 2 的幂时会有轻微偏差
                // 注意当范围为 MIN..=MAX 时会溢出, 此时应该调用随机数生成方法而非范围方法
                let random_val = rng.$rng_method() as $ty % (range + 1);
                start + random_val
            }
        }
    };
}

impl_unsigned_sample_range!(u8, next_byte);
impl_unsigned_sample_range!(u16, next_u32);
impl_unsigned_sample_range!(u32, next_u32);
impl_unsigned_sample_range!(u64, next_u64);
#[cfg(target_pointer_width = "32")]
impl_unsigned_sample_range!(usize, next_u32);
#[cfg(target_pointer_width = "64")]
impl_unsigned_sample_range!(usize, next_u64);

macro_rules! impl_signed_sample_range {
    ($ty:ty, $rng_method:ident, $unsigned_ty:ty, $xor_mask:expr) => {
        impl SampleRange for std::ops::Range<$ty> {
            type Output = $ty;

            fn sample<R: RngCore>(&self, rng: &mut R) -> $ty {
                if self.start == self.end {
                    return self.start;
                }

                // 我们通过等距变换先映射到对应的无符号整型, 此时的运算不会溢出
                let start = (self.start as $unsigned_ty) ^ $xor_mask;
                let end = (self.end as $unsigned_ty) ^ $xor_mask;
                let range = end - start;

                // 注意这不是无偏的, 模运算的余数出现概率在模数不是 2 的幂时会有轻微偏差
                let random_val = rng.$rng_method() as $unsigned_ty % range;
                // 最后再等距映射回对应的有符号整型
                ((start + random_val) ^ $xor_mask) as $ty
            }
        }

        impl SampleRange for std::ops::RangeInclusive<$ty> {
            type Output = $ty;

            fn sample<R: RngCore>(&self, rng: &mut R) -> $ty {
                let start_val = *self.start();
                let end_val = *self.end();

                if start_val == end_val {
                    return start_val;
                }

                let start = (start_val as $unsigned_ty) ^ $xor_mask;
                let end = (end_val as $unsigned_ty) ^ $xor_mask;
                let range = end - start;

                debug_assert_ne!(
                    range,
                    <$unsigned_ty>::MAX,
                    "RangeInclusive<MIN..=MAX> would overflow!"
                );

                // 注意这不是无偏的, 模运算的余数出现概率在模数不是 2 的幂时会有轻微偏差
                // 注意当范围为 MIN..=MAX 时会溢出, 此时应该调用随机数生成方法而非范围方法
                let random_val = rng.$rng_method() as $unsigned_ty % (range + 1);
                ((start + random_val) ^ $xor_mask) as $ty
            }
        }
    };
}

impl_signed_sample_range!(i8, next_byte, u8, 1u8 << 7);
impl_signed_sample_range!(i16, next_u32, u16, 1u16 << 15);
impl_signed_sample_range!(i32, next_u32, u32, 1u32 << 31);
impl_signed_sample_range!(i64, next_u64, u64, 1u64 << 63);
#[cfg(target_pointer_width = "32")]
impl_signed_sample_range!(isize, next_u32, u32, 1u32 << 31);
#[cfg(target_pointer_width = "64")]
impl_signed_sample_range!(isize, next_u64, u64, 1u64 << 63);

fn uniform_f64<R: RngCore>(rng: &mut R) -> f64 {
    let mut bytes = [0u8; 8];
    rng.fill_bytes(&mut bytes);
    let value = u64::from_le_bytes(bytes);

    // 使用 53 位精度 (IEEE 754 double 的尾数位数)
    const MANTISSA_BITS: u64 = 53;
    const MANTISSA_MASK: u64 = (1 << MANTISSA_BITS) - 1;

    let mantissa = value & MANTISSA_MASK;
    let scale = (1u64 << MANTISSA_BITS) as f64;

    mantissa as f64 / scale
}

impl SampleRange for Range<f64> {
    type Output = f64;

    /// Full floating-point range is not supported due to floating-point precision.
    fn sample<R: RngCore>(&self, rng: &mut R) -> f64 {
        let min = self.start;
        let max = self.end;
        let uniform = uniform_f64(rng);
        min + uniform * (max - min)
    }
}

impl SampleRange for RangeInclusive<f64> {
    type Output = f64;

    /// Full floating-point range is not supported due to floating-point precision.
    fn sample<R: RngCore>(&self, rng: &mut R) -> f64 {
        let min = *self.start();
        let max = *self.end();
        let uniform = uniform_f64(rng);
        min + uniform * (max - min)
    }
}
