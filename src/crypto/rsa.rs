//! Separate implementation of Pkcs1v15Encrypt RSA encryption only to omit dependencies on external RSA Crates
//! Since the vanilla RSA relies on rand 0.6, this is implemented in order to upgrade the rand

use crate::crypto::rand::Rng;

use super::biguint::BigUint;

/// RSA PKCS#1 v1.5 instance
pub struct RsaPkcs1v15 {
    n: BigUint, // 模数
    e: BigUint, // 指数
}

impl RsaPkcs1v15 {
    /// Create RsaPkcs1v15 from PEM formatted public key
    pub fn from_pem(pem: &str) -> Self {
        // 提取 Base64 部分
        let base64_str = pem
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect::<String>();

        // 解码 DER 格式
        let der = crate::crypto::decode_base64(base64_str);

        // 公钥的 ASN.1 结构通常是: SEQUENCE { SEQUENCE { OID, NULL }, BITSTRING }
        let mut cursor = 0;

        // 读取外层 SEQUENCE
        #[cfg(debug_assertions)]
        assert_eq!(der[cursor], 0x30, "Expected SEQUENCE");
        cursor += 1;
        let _ = Self::read_length(&der, &mut cursor); // 跳过序列长度

        // 读取内层 SEQUENCE (算法标识符)
        #[cfg(debug_assertions)]
        assert_eq!(der[cursor], 0x30, "Expected inner SEQUENCE");
        cursor += 1;
        let seq_len = Self::read_length(&der, &mut cursor);
        // 记录 SEQUENCE 的结束位置
        let _seq_end = cursor + seq_len;

        // 跳过算法 OID (rsaEncryption: 1.2.840.113549.1.1.1)
        cursor += seq_len;

        // 读取 BITSTRING
        #[cfg(debug_assertions)]
        assert_eq!(der[cursor], 0x03, "Expected BITSTRING");
        cursor += 1;
        // 读取 BITSTRING 的长度
        let _bitstring_len = Self::read_length(&der, &mut cursor);

        // BITSTRING 第一个字节是未使用位数 (通常为0)
        #[cfg(debug_assertions)]
        assert_eq!(der[cursor], 0x00, "Expected 0 unused bits");
        cursor += 1;

        // 现在读取实际的公钥数据 (又一个 SEQUENCE)
        #[cfg(debug_assertions)]
        assert_eq!(der[cursor], 0x30, "Expected SEQUENCE for public key");
        cursor += 1;
        let _ = Self::read_length(&der, &mut cursor); // 跳过序列长度

        // 读取模数 n
        #[cfg(debug_assertions)]
        assert_eq!(der[cursor], 0x02, "Expected INTEGER for modulus");
        cursor += 1;
        let n_len = Self::read_length(&der, &mut cursor);
        let n_bytes = &der[cursor..cursor + n_len];
        cursor += n_len;

        // 读取指数 e
        #[cfg(debug_assertions)]
        assert_eq!(der[cursor], 0x02, "Expected INTEGER for exponent");
        cursor += 1;
        let e_len = Self::read_length(&der, &mut cursor);
        let e_bytes = &der[cursor..cursor + e_len];

        RsaPkcs1v15 {
            n: BigUint::from_bytes_be(n_bytes),
            e: BigUint::from_bytes_be(e_bytes),
        }
    }

    // 辅助函数: 读取 DER 长度字段
    fn read_length(der: &[u8], cursor: &mut usize) -> usize {
        let mut len = der[*cursor] as usize;
        *cursor += 1;

        if len & 0x80 != 0 {
            // 长格式长度
            let num_bytes = len & 0x7F;
            len = 0;
            for _ in 0..num_bytes {
                len = (len << 8) | der[*cursor] as usize;
                *cursor += 1;
            }
        }
        len
    }

    fn pkcs1v15_pad(&self, rng: &mut impl Rng, data: &[u8]) -> Vec<u8> {
        // PKCS#1 v1.5 加密填充
        let k = (self.n.bits() / 8) as usize;
        let mut padded = vec![0u8; k];
        // 第一个字节是 0x00
        // 第二个字节是 0x02 (加密块)
        padded[1] = 0x02;
        // 填充随机非零字节
        let ps_len = k - data.len() - 3;
        for i in 0..ps_len {
            padded[2 + i] = rng.random_range(1u8..=255);
        }
        // 分隔符 0x00
        padded[2 + ps_len] = 0x00;
        // 复制原始数据
        padded[3 + ps_len..].copy_from_slice(data);
        padded
    }

    /// Encrypt data using RSA with PKCS#1 v1.5 padding
    pub fn encrypt(&self, rng: &mut impl Rng, data: &[u8]) -> Vec<u8> {
        // RSA 加密: ciphertext = plaintext^e mod n

        // 确保数据长度合适 (PKCS#1 v1.5)
        // TODO 也许这里应该进行一下错误处理
        #[cfg(debug_assertions)]
        assert!(
            data.len() <= (self.n.bits() / 8 - 11) as usize,
            "Data too long for RSA modulus"
        );

        // 应用 PKCS#1 v1.5 填充
        let padded = self.pkcs1v15_pad(rng, data);

        // 将填充后的数据转换为大整数
        let m = BigUint::from_bytes_be(&padded);

        // 计算 m^e mod n
        let c = m.modpow(&self.e, &self.n);

        // 返回大整数的大端字节表示
        c.to_bytes_be()
    }
}
