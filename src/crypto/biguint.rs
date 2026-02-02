//! Self-implemented BigUint for RSA should not be used anywhere else.
//! In the case of only targeting the common RSA public exponent (65537), this implementation is about 50% faster than num_bigint
//! **Warning**: Not constant-time, may become a side-channel attack vector. Use only in trusted environments.
//! Example for use in transmitting doubly-encrypted Token over HTTPS

// 在仅针对 RSA 常见公钥指数 (65537) 的情况下, 该实现比 num_bigint 快约 50%
// 警告: 非常数时间, 可能成为一个侧信道攻击向量. 仅在受信任环境中使用.
// 如仅用于在 HTTPS 中传输 二次加密的 Token

type BigDigit = u64;
const BITS: u64 = 64;

/// The BigUint implementation for RSA should not be used anywhere else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigUint {
    // 数据以小端序存储: data[0] 是最低 64 位
    data: Vec<BigDigit>,
}

impl BigUint {
    // === Constructors ===

    /// Create zero BigUint
    pub fn zero() -> Self {
        Self { data: vec![0] }
    }

    /// Create one BigUint
    pub fn one() -> Self {
        Self { data: vec![1] }
    }

    /// 从大端序字节数组创建 (解析 PEM)
    pub fn from_bytes_be(bytes: &[u8]) -> Self {
        let mut data = Vec::with_capacity((bytes.len() + 7) / 8);
        for chunk in bytes.rchunks(8) {
            let mut word = 0u64;
            for byte in chunk {
                word = (word << 8) | (*byte as u64);
            }
            data.push(word);
        }
        let mut res = Self { data };
        res.normalize();
        res
    }

    /// 转换为大端序字节数组 (输出密文)
    pub fn to_bytes_be(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        if self.data.is_empty() {
            return vec![0];
        }
        // 既然要转成字节流, 我们先提取所有完整字节, 最后再处理可能的前导零
        // 但为了简单, 我们直接生成所有字节, 然后去除前导零
        for &word in self.data.iter().rev() {
            bytes.extend_from_slice(&word.to_be_bytes());
        }

        // 去除结果中的前导零 (大端序的开头)
        let first_nonzero = bytes
            .iter()
            .position(|&x| x != 0)
            .unwrap_or(bytes.len() - 1);
        bytes[first_nonzero..].to_vec()
    }

    /// 返回比特数 (用于 RSA 填充计算)
    pub fn bits(&self) -> u64 {
        if self.data.is_empty() {
            return 0;
        }
        let last_idx = self.data.len() - 1;
        let last_word = self.data[last_idx];
        let last_bits = BITS - last_word.leading_zeros() as u64;
        (last_idx as u64 * BITS) + last_bits
    }

    // === Internal Helpers ===

    // 去除高位零
    fn normalize(&mut self) {
        while self.data.len() > 1 && self.data.last() == Some(&0) {
            self.data.pop();
        }
    }

    // 判断 self >= other
    fn ge(&self, other: &Self) -> bool {
        if self.data.len() != other.data.len() {
            return self.data.len() > other.data.len();
        }
        for (a, b) in self.data.iter().rev().zip(other.data.iter().rev()) {
            if a != b {
                return a > b;
            }
        }
        true // equal
    }

    // === Basic Operations (only for Montgomery algorithm) ===

    // self = self - other. 假设 self >= other.
    fn sub_assign(&mut self, other: &Self) {
        let mut borrow = 0u64;
        for i in 0..self.data.len() {
            let lhs = self.data[i];
            let rhs = if i < other.data.len() {
                other.data[i]
            } else {
                0
            };

            let (diff, b1) = lhs.overflowing_sub(rhs);
            let (diff, b2) = diff.overflowing_sub(borrow);
            self.data[i] = diff;
            borrow = (if b1 { 1 } else { 0 }) + (if b2 { 1 } else { 0 });
        }
        self.normalize();
    }

    // self = self << 1
    fn shl1_assign(&mut self) {
        let mut carry = 0;
        for word in self.data.iter_mut() {
            let next_carry = *word >> 63;
            *word = (*word << 1) | carry;
            carry = next_carry;
        }
        if carry > 0 {
            self.data.push(carry);
        }
    }

    // 蒙哥马利核心算法
    // 计算 n0', 使得 (n0 * n0') = -1 mod 2^64
    fn mont_inv_digit(n0: u64) -> u64 {
        let mut inv = 1u64;
        for _ in 0..63 {
            // Newton-Raphson 迭代
            inv = inv.wrapping_mul(2u64.wrapping_sub(n0.wrapping_mul(inv)));
        }
        inv.wrapping_neg()
    }

    // 蒙哥马利乘法: res = x * y * R^-1 mod m
    // 这里 R = 2^(64 * num_words)
    fn mont_mul(x: &Self, y: &Self, m: &Self, inv: u64, num_words: usize) -> Self {
        // 结果缓冲区，大小为 2*n + 1 以防溢出
        let mut t = vec![0u64; num_words * 2 + 1];

        // 标准乘法 x * y
        for (i, &xi) in x.data.iter().enumerate() {
            let mut carry = 0u64;
            for (j, &yj) in y.data.iter().enumerate() {
                let product = (xi as u128 * yj as u128) + (t[i + j] as u128) + (carry as u128);
                t[i + j] = product as u64;
                carry = (product >> 64) as u64;
            }
            let mut k = i + y.data.len();
            while carry > 0 {
                let sum = (t[k] as u128) + (carry as u128);
                t[k] = sum as u64;
                carry = (sum >> 64) as u64;
                k += 1;
            }
        }

        // 蒙哥马利约减 (Reduction)
        // 使得 t 变为 t * R^-1 mod m
        for i in 0..num_words {
            // u = t[i] * inv mod 2^64
            let u = t[i].wrapping_mul(inv);

            // t = t + u * m * 2^(i*64)
            // 实际上只需要处理 m 的 word，加到 t[i...] 上
            let mut carry = 0u64;
            for (j, &mj) in m.data.iter().enumerate() {
                let product = (u as u128 * mj as u128) + (t[i + j] as u128) + (carry as u128);
                t[i + j] = product as u64;
                carry = (product >> 64) as u64;
            }

            // 处理 m 长度之外的进位
            let mut k = i + num_words;
            while carry > 0 {
                let sum = (t[k] as u128) + (carry as u128);
                t[k] = sum as u64;
                carry = (sum >> 64) as u64;
                k += 1;
            }
        }

        // 结果是 t[num_words..]
        // 理论上 t[0..num_words] 现在应该全是 0
        let mut res = BigUint {
            data: t[num_words..].to_vec(),
        };
        res.normalize();

        // 最后的条件减法: if res >= m { res -= m }
        if res.ge(m) {
            res.sub_assign(m);
        }
        res
    }

    /// res = base^exp mod m
    pub fn modpow(&self, exp: &Self, modulus: &Self) -> Self {
        // TODO: 边界检查, 或应 panic
        if modulus.data.is_empty() {
            return Self::zero();
        }

        let num_words = modulus.data.len();

        // 计算 Montgomery 参数
        // inv = -m[0]^-1 mod 2^64
        let inv = Self::mont_inv_digit(modulus.data[0]);

        // 预计算 R^2 mod m
        // 左移-减法, 避免除法
        // R = 2^(num_words * 64)
        // 初始 rr = 1
        let mut rr = BigUint::one();

        // 我们需要左移 2 * num_words * 64 次
        // 每次左移后, 如果 >= modulus, 就减去 modulus
        let target_bits = (num_words as u64) * 64 * 2;

        // TODO: 优化: R (即 1 << num_words*64) 肯定是比 m 大的
        // 为保持代码极简且正确, 这里用朴素的逐位 shift. 对于 RSA 2048, 仅几千次循环
        for _ in 0..target_bits {
            rr.shl1_assign();
            if rr.ge(modulus) {
                rr.sub_assign(modulus);
            }
        }

        // 将 base 转换到蒙哥马利域: base_mont = base * R mod m
        // mont_mul(base, R^2, m) = base * R^2 * R^-1 = base * R
        let base_mont = Self::mont_mul(self, &rr, modulus, inv, num_words);

        // 初始化结果为 1 的蒙哥马利形式: res_mont = 1 * R mod m
        // 即 rr 的第一部分: mont_mul(1, R^2, m)
        let one = BigUint::one();

        // 比 num_bigint 快约 50%
        // 针对绝大多数情况 RSA 公钥指数 65537 (0x10001 = 2^16 + 1) 仅需 17 次 Montgomery 乘法
        if exp.data.len() == 1 && exp.data[0] == 65537 {
            // 从 base 开始累乘
            // 16 次平方: base^(2^16) = base^65536
            let mut res_mont = base_mont.clone();
            for _ in 0..16 {
                res_mont = Self::mont_mul(&res_mont, &res_mont, modulus, inv, num_words);
            }

            // 1 次乘法: base^65536 * base = base^65537
            res_mont = Self::mont_mul(&res_mont, &base_mont, modulus, inv, num_words);

            // 转回普通域: res = res_mont * R^-1 mod m
            return Self::mont_mul(&res_mont, &one, modulus, inv, num_words);
        }

        // 比 num_bigint 慢约 45%
        // 通用情况: 任意指数, 从 1 开始累乘
        let mut res_mont = Self::mont_mul(&one, &rr, modulus, inv, num_words);

        // 滑动窗口或简单的二进制指数法
        // 这里使用简单的二进制位遍历 (从高位到低位)
        // 为了方便, 先找到最高有效位
        let exp_bits = exp.bits();
        if exp_bits == 0 {
            return BigUint::one(); // m^0 = 1
        }

        for i in (0..exp_bits).rev() {
            // 平方: res = res * res
            res_mont = Self::mont_mul(&res_mont, &res_mont, modulus, inv, num_words);

            // 如果该位是 1, 乘法: res = res * base
            // 获取 exp 的第 i 位
            let word_idx = (i / 64) as usize;
            let bit_idx = (i % 64) as u8;
            if (exp.data[word_idx] >> bit_idx) & 1 == 1 {
                res_mont = Self::mont_mul(&res_mont, &base_mont, modulus, inv, num_words);
            }
        }

        // 转回普通域: res = res_mont * R^-1 mod m
        // mont_mul(res_mont, 1, m) = res * R * 1 * R^-1 = res
        Self::mont_mul(&res_mont, &one, modulus, inv, num_words)
    }
}
