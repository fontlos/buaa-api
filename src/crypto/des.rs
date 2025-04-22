use des::Des;
use des::cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};

/// DES encrypt, use ECB mode
pub fn des_encrypt(data: &str, key: &str) -> String {
    // 
    let key = key.as_bytes();
    let cipher = Des::new_from_slice(key).unwrap();
    let mut input = data.as_bytes().to_vec();
    // 计算填充长度
    let padding_len = 8 - input.len() % 8;
    for _ in 0..padding_len {
        input.push(padding_len as u8);
    }
    // 创建输出缓冲区
    let mut output = vec![0u8; input.len()];
    for (i, chunk) in input.chunks(8).enumerate() {
        let mut block = GenericArray::clone_from_slice(chunk);
        cipher.encrypt_block(&mut block);
        output[i * 8..(i + 1) * 8].copy_from_slice(&block);
    }
    // 将加密后的数据转换为 Hex 格式
    #[cfg(feature = "crypto")]
    return hex::encode(&output);
    #[cfg(not(feature = "crypto"))]
    super::byte2hex::bytes_to_hex_fast(&output)
}
