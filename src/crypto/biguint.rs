//! Self-implemented BigUint for RSA should not be used anywhere else.

type BigDigit = u64;
type DoubleBigDigit = u128;
const BITS: u8 = 64;

/// The BigUint implementation for RSA should not be used anywhere else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigUint {
    data: Vec<BigDigit>,
}

impl BigUint {
    /// 从大端字节序创建 BigUint (内部存储为小端)
    pub fn from_bytes_be(bytes: &[u8]) -> BigUint {
        if bytes.is_empty() {
            return BigUint { data: Vec::new() };
        }
        // 反转字节，变为小端
        let mut bytes = bytes.to_vec();
        bytes.reverse();

        // 下面开始从小端字节序列创建
        let mut data = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            let mut chunk = [0u8; 8];
            let chunk_len = std::cmp::min(8, bytes.len() - i);
            chunk[..chunk_len].copy_from_slice(&bytes[i..i + chunk_len]);
            data.push(u64::from_le_bytes(chunk));
            i += chunk_len;
        }
        // 去除高位的前导零
        while let Some(&last) = data.last() {
            if last == 0 {
                data.pop();
            } else {
                break;
            }
        }
        BigUint { data }
    }

    /// 转换为大端字节序
    pub fn to_bytes_be(&self) -> Vec<u8> {
        // 先转换为小端字节序列
        if self.data.is_empty() {
            return vec![0];
        }
        let mut bytes = Vec::new();
        for &digit in &self.data {
            bytes.extend_from_slice(&digit.to_le_bytes());
        }
        // 去除高位的前导零字节
        while let Some(&last) = bytes.last() {
            if last == 0 {
                bytes.pop();
            } else {
                break;
            }
        }

        // 反转为大端
        bytes.reverse();
        bytes
    }

    /// 返回所需的位数
    pub fn bits(&self) -> u64 {
        if self.data.is_empty() {
            return 0;
        }
        let last = *self.data.last().unwrap();
        64 * (self.data.len() as u64 - 1) + (64 - last.leading_zeros() as u64)
    }

    /// 幂模运算 self^exp mod modulus
    pub fn modpow(&self, exp: &BigUint, modulus: &BigUint) -> BigUint {
        monty::monty_modpow(self, exp, modulus)
    }
}

// 基本方法
impl BigUint {
    const ZERO: BigUint = BigUint { data: Vec::new() };

    fn one() -> Self {
        BigUint { data: vec![1] }
    }

    #[inline]
    fn normalize(&mut self) {
        if let Some(&0) = self.data.last() {
            let len = self.data.iter().rposition(|&d| d != 0).map_or(0, |i| i + 1);
            self.data.truncate(len);
        }
        if self.data.len() < self.data.capacity() / 4 {
            self.data.shrink_to_fit();
        }
    }

    #[inline]
    fn normalized(mut self) -> BigUint {
        self.normalize();
        self
    }

    // 左移位运算
    pub fn shl(&self, bits: u64) -> Self {
        if self.data.is_empty() {
            return BigUint::ZERO;
        }

        let word_shift = (bits / 64) as usize;
        let bit_shift = (bits % 64) as u32;

        let mut data = vec![0; word_shift];
        data.extend_from_slice(&self.data);

        if bit_shift > 0 {
            let mut carry = 0;
            for digit in &mut data[word_shift..] {
                let new_carry = *digit >> (64 - bit_shift);
                *digit = (*digit << bit_shift) | carry;
                carry = new_carry;
            }
            if carry != 0 {
                data.push(carry);
            }
        }

        BigUint { data }
    }

    // 右移位运算
    fn shr(&self, bits: u64) -> BigUint {
        if self.data.is_empty() {
            return BigUint::ZERO;
        }

        let word_shift = (bits / 64) as usize;
        let bit_shift = (bits % 64) as u32;

        if word_shift >= self.data.len() {
            return BigUint::ZERO;
        }

        let mut data = if word_shift > 0 {
            self.data[word_shift..].to_vec()
        } else {
            self.data.clone()
        };

        if bit_shift > 0 {
            let mut carry = 0;
            for digit in data.iter_mut().rev() {
                let new_carry = *digit << (64 - bit_shift);
                *digit = (*digit >> bit_shift) | carry;
                carry = new_carry;
            }
        }

        BigUint { data }.normalized()
    }
}

mod monty {
    //! 来自 num_bigint crate

    use super::{BITS, BigDigit, BigUint, DoubleBigDigit};

    struct MontyReducer {
        n0inv: BigDigit,
    }

    impl MontyReducer {
        fn new(n: &BigUint) -> Self {
            let n0inv = Self::inv_mod_alt(n.data[0]);
            MontyReducer { n0inv }
        }

        // k0 = -m**-1 mod 2**BITS. Algorithm from: Dumas, J.G. "On Newton–Raphson
        // Iteration for Multiplicative Inverses Modulo Prime Powers".
        fn inv_mod_alt(b: BigDigit) -> BigDigit {
            assert_ne!(b & 1, 0);

            let mut k0 = BigDigit::wrapping_sub(2, b);
            let mut t = b - 1;
            let mut i = 1;
            while i < BITS {
                t = t.wrapping_mul(t);
                k0 = k0.wrapping_mul(t + 1);

                i <<= 1;
            }
            debug_assert_eq!(k0.wrapping_mul(b), 1);
            k0.wrapping_neg()
        }
    }

    // 取模运算只发生在对 RSA 模数取模上, 这个数通常很大, 无需考虑边界情况直接对大数取模即可
    pub(super) fn monty_modpow(x: &BigUint, y: &BigUint, m: &BigUint) -> BigUint {
        assert!(m.data[0] & 1 == 1);
        let mr = MontyReducer::new(m);
        let num_words = m.data.len();

        let mut x = x.clone();

        // We want the lengths of x and m to be equal.
        // It is OK if x >= m as long as len(x) == len(m).
        if x.data.len() > num_words {
            x %= m;
            // Note: now len(x) <= numWords, not guaranteed ==.
        }
        if x.data.len() < num_words {
            x.data.resize(num_words, 0);
        }

        // rr = 2**(2*_W*len(m)) mod m
        let mut rr = BigUint::one();
        rr = (rr.shl(2 * num_words as u64 * u64::from(BITS))) % m;
        if rr.data.len() < num_words {
            rr.data.resize(num_words, 0);
        }
        // one = 1, with equal length to that of m
        let mut one = BigUint::one();
        one.data.resize(num_words, 0);

        let n = 4;
        // powers[i] contains x^i
        let mut powers = Vec::with_capacity(1 << n);
        powers.push(montgomery(&one, &rr, m, mr.n0inv, num_words));
        powers.push(montgomery(&x, &rr, m, mr.n0inv, num_words));
        for i in 2..1 << n {
            let r = montgomery(&powers[i - 1], &powers[1], m, mr.n0inv, num_words);
            powers.push(r);
        }

        // initialize z = 1 (Montgomery 1)
        let mut z = powers[0].clone();
        z.data.resize(num_words, 0);
        let mut zz = BigUint::ZERO;
        zz.data.resize(num_words, 0);

        // same windowed exponent, but with Montgomery multiplications
        for i in (0..y.data.len()).rev() {
            let mut yi = y.data[i];
            let mut j = 0;
            while j < BITS {
                if i != y.data.len() - 1 || j != 0 {
                    zz = montgomery(&z, &z, m, mr.n0inv, num_words);
                    z = montgomery(&zz, &zz, m, mr.n0inv, num_words);
                    zz = montgomery(&z, &z, m, mr.n0inv, num_words);
                    z = montgomery(&zz, &zz, m, mr.n0inv, num_words);
                }
                zz = montgomery(
                    &z,
                    &powers[(yi >> (BITS - n)) as usize],
                    m,
                    mr.n0inv,
                    num_words,
                );
                core::mem::swap(&mut z, &mut zz);
                yi <<= n;
                j += n;
            }
        }

        // convert to regular number
        zz = montgomery(&z, &one, m, mr.n0inv, num_words);

        zz.normalize();
        // One last reduction, just in case.
        // See golang.org/issue/13907.
        if zz >= *m {
            // Common case is m has high bit set; in that case,
            // since zz is the same length as m, there can be just
            // one multiple of m to remove. Just subtract.
            // We think that the subtract should be sufficient in general,
            // so do that unconditionally, but double-check,
            // in case our beliefs are wrong.
            // The div is not expected to be reached.
            zz -= m;
            if zz >= *m {
                zz %= m;
            }
        }

        zz.normalize();
        zz
    }

    fn montgomery(x: &BigUint, y: &BigUint, m: &BigUint, k: BigDigit, n: usize) -> BigUint {
        // This code assumes x, y, m are all the same length, n.
        // (required by addMulVVW and the for loop).
        // It also assumes that x, y are already reduced mod m,
        // or else the result will not be properly reduced.
        assert!(
            x.data.len() == n && y.data.len() == n && m.data.len() == n,
            "{:?} {:?} {:?} {}",
            x,
            y,
            m,
            n
        );

        let mut z = BigUint::ZERO;
        z.data.resize(n * 2, 0);

        let mut c: BigDigit = 0;
        for i in 0..n {
            let c2 = add_mul_vvw(&mut z.data[i..n + i], &x.data, y.data[i]);
            let t = z.data[i].wrapping_mul(k);
            let c3 = add_mul_vvw(&mut z.data[i..n + i], &m.data, t);
            let cx = c.wrapping_add(c2);
            let cy = cx.wrapping_add(c3);
            z.data[n + i] = cy;
            if cx < c2 || cy < c3 {
                c = 1;
            } else {
                c = 0;
            }
        }

        if c == 0 {
            z.data = z.data[n..].to_vec();
        } else {
            {
                let (first, second) = z.data.split_at_mut(n);
                sub_vv(first, second, &m.data);
            }
            z.data = z.data[..n].to_vec();
        }

        z
    }

    /// The resulting carry c is either 0 or 1.
    #[inline(always)]
    fn sub_vv(z: &mut [BigDigit], x: &[BigDigit], y: &[BigDigit]) -> BigDigit {
        let mut c = 0;
        for (i, (xi, yi)) in x.iter().zip(y.iter()).enumerate().take(z.len()) {
            let zi = xi.wrapping_sub(*yi).wrapping_sub(c);
            z[i] = zi;
            // see "Hacker's Delight", section 2-12 (overflow detection)
            c = ((yi & !xi) | ((yi | !xi) & zi)) >> (BITS - 1)
        }

        c
    }

    #[inline(always)]
    fn add_mul_vvw(z: &mut [BigDigit], x: &[BigDigit], y: BigDigit) -> BigDigit {
        let mut c = 0;
        for (zi, xi) in z.iter_mut().zip(x.iter()) {
            let (z1, z0) = mul_add_www(*xi, y, *zi);
            let (c_, zi_) = add_ww(z0, c, 0);
            *zi = zi_;
            c = c_ + z1;
        }

        c
    }

    /// z1<<_W + z0 = x+y+c, with c == 0 or 1
    #[inline(always)]
    fn add_ww(x: BigDigit, y: BigDigit, c: BigDigit) -> (BigDigit, BigDigit) {
        let yc = y.wrapping_add(c);
        let z0 = x.wrapping_add(yc);
        let z1 = if z0 < x || yc < y { 1 } else { 0 };

        (z1, z0)
    }

    /// z1 << _W + z0 = x * y + c
    #[inline(always)]
    fn mul_add_www(x: BigDigit, y: BigDigit, c: BigDigit) -> (BigDigit, BigDigit) {
        let z = x as DoubleBigDigit * y as DoubleBigDigit + c as DoubleBigDigit;
        ((z >> BITS) as BigDigit, z as BigDigit)
    }
}

mod rem {
    //! 针对 RSA 的模运算, 使用长除法.
    //! 做了部分假设, 默认两数不为零且被除数大于等于除数.
    //! 仅在 monty_modpow 函数中使用
    use super::{BigDigit, BigUint};
    use core::ops::{Rem, RemAssign};

    impl Rem<&BigUint> for &BigUint {
        type Output = BigUint;

        fn rem(self, other: &BigUint) -> BigUint {
            let m = self.data.len();
            let n = other.data.len();

            debug_assert!(!other.data.is_empty());
            debug_assert!(!self.data.is_empty());
            debug_assert!(m >= n);

            // 标准化除数，使得最高位 >= BASE/2
            let shift = other.data.last().unwrap().leading_zeros();
            let divisor = if shift > 0 {
                other.shl(shift as u64)
            } else {
                other.clone()
            };

            let dividend = if shift > 0 {
                self.shl(shift as u64)
            } else {
                self.clone()
            };

            let mut remainder = dividend;

            for j in (0..=m - n).rev() {
                // 估算商的一位
                let r_high = *remainder.data.get(j + n).unwrap_or(&0);
                let r_mid = remainder.data[j + n - 1];
                let r_low = remainder.data.get(j + n - 2).copied().unwrap_or(0);

                let divisor_high = divisor.data[n - 1];
                let divisor_mid = divisor.data.get(n - 2).copied().unwrap_or(0);

                // 估算商
                let mut q_hat = if r_high == divisor_high {
                    BigDigit::MAX
                } else {
                    let num = ((r_high as u128) << 64) | (r_mid as u128);
                    (num / divisor_high as u128) as u64
                };

                // 调整商
                while q_hat > 0 {
                    let product_high = (q_hat as u128) * (divisor_mid as u128);
                    let num_high = ((r_high as u128) << 64) | (r_mid as u128);
                    let num_low = ((r_mid as u128) << 64) | (r_low as u128);

                    if product_high > num_high {
                        q_hat -= 1;
                        continue;
                    }

                    if product_high == num_high {
                        let product_low = (q_hat as u128) * (divisor_mid as u128);
                        if product_low > num_low {
                            q_hat -= 1;
                            continue;
                        }
                    }
                    break;
                }

                // 乘法和减法 - 修复溢出问题
                let mut borrow: u64 = 0;
                for i in 0..n {
                    let product = (q_hat as u128) * (divisor.data[i] as u128);
                    let product_high = (product >> 64) as u64;
                    let product_low = product as u64;

                    // 先加上之前的借位
                    let (sum1, carry1) = product_low.overflowing_add(borrow);

                    // 然后从被除数中减去
                    let (diff, borrow1) = remainder.data[j + i].overflowing_sub(sum1);
                    remainder.data[j + i] = diff;

                    // 计算新的借位
                    borrow =
                        product_high + if carry1 { 1 } else { 0 } + if borrow1 { 1 } else { 0 };
                }

                // 处理最高位的借位
                if j + n < remainder.data.len() {
                    let (diff, borrow1) = remainder.data[j + n].overflowing_sub(borrow);
                    remainder.data[j + n] = diff;
                    borrow = if borrow1 { 1 } else { 0 };
                }

                // 如果减法导致借位，调整商
                if borrow > 0 {
                    // 加回除数
                    let mut carry: u64 = 0;
                    for i in 0..n {
                        let sum = (remainder.data[j + i] as u128)
                            + (divisor.data[i] as u128)
                            + (carry as u128);
                        remainder.data[j + i] = sum as u64;
                        carry = (sum >> 64) as u64;
                    }

                    if j + n < remainder.data.len() {
                        remainder.data[j + n] = remainder.data[j + n].wrapping_add(carry);
                    }
                }
            }

            // 反标准化余数
            let remainder = if shift > 0 {
                let mut rem = BigUint {
                    data: remainder.data,
                };
                rem.normalize();
                rem.shr(shift as u64)
            } else {
                BigUint {
                    data: remainder.data,
                }
                .normalized()
            };
            remainder
        }
    }

    impl Rem<&BigUint> for BigUint {
        type Output = BigUint;

        #[inline]
        fn rem(self, other: &BigUint) -> BigUint {
            Rem::rem(&self, other)
        }
    }

    impl RemAssign<&BigUint> for BigUint {
        #[inline]
        fn rem_assign(&mut self, other: &BigUint) {
            *self = &*self % other;
        }
    }
}

mod sub {
    //! 来自 num_bigint crate

    use super::{BigDigit, BigUint};
    use core::ops::SubAssign;

    impl SubAssign<&BigUint> for BigUint {
        fn sub_assign(&mut self, other: &BigUint) {
            sub(&mut self.data[..], &other.data[..]);
            self.normalize();
        }
    }

    pub(super) fn sub(a: &mut [BigDigit], b: &[BigDigit]) {
        let mut borrow = 0;

        let len = Ord::min(a.len(), b.len());
        let (a_lo, a_hi) = a.split_at_mut(len);
        let (b_lo, b_hi) = b.split_at(len);

        for (a, b) in a_lo.iter_mut().zip(b_lo) {
            borrow = sbb(borrow, *a, *b, a);
        }

        if borrow != 0 {
            for a in a_hi {
                borrow = sbb(borrow, *a, 0, a);
                if borrow == 0 {
                    break;
                }
            }
        }

        // note: we're _required_ to fail on underflow
        assert!(
            borrow == 0 && b_hi.iter().all(|x| *x == 0),
            "Cannot subtract b from a because b is larger than a."
        );
    }

    // Subtract with borrow:
    #[cfg(target_arch = "x86_64")]
    #[inline]
    fn sbb(borrow: u8, a: u64, b: u64, out: &mut u64) -> u8 {
        // Safety: There are absolutely no safety concerns with calling `_subborrow_u64`.
        // It's just unsafe for API consistency with other intrinsics.
        unsafe { core::arch::x86_64::_subborrow_u64(borrow, a, b, out) }
    }

    #[cfg(target_arch = "x86")]
    #[inline]
    fn sbb(borrow: u8, a: u32, b: u32, out: &mut u32) -> u8 {
        // Safety: There are absolutely no safety concerns with calling `_subborrow_u32`.
        // It's just unsafe for API consistency with other intrinsics.
        unsafe { core::arch::x86::_subborrow_u32(borrow, a, b, out) }
    }

    // fallback for environments where we don't have a subborrow intrinsic
    // (copied from the standard library's `borrowing_sub`)
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline]
    fn sbb(borrow: u8, lhs: BigDigit, rhs: BigDigit, out: &mut BigDigit) -> u8 {
        let (a, b) = lhs.overflowing_sub(rhs);
        let (c, d) = a.overflowing_sub(borrow as BigDigit);
        *out = c;
        u8::from(b || d)
    }
}

mod cmp {
    use super::{BigDigit, BigUint};
    use core::cmp::Ordering;

    impl PartialOrd for BigUint {
        #[inline]
        fn partial_cmp(&self, other: &BigUint) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for BigUint {
        #[inline]
        fn cmp(&self, other: &BigUint) -> Ordering {
            cmp_slice(&self.data[..], &other.data[..])
        }
    }

    #[inline]
    fn cmp_slice(a: &[BigDigit], b: &[BigDigit]) -> Ordering {
        debug_assert!(a.last() != Some(&0));
        debug_assert!(b.last() != Some(&0));

        match Ord::cmp(&a.len(), &b.len()) {
            Ordering::Equal => Iterator::cmp(a.iter().rev(), b.iter().rev()),
            other => other,
        }
    }
}
