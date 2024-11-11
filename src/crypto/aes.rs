use aes::Aes128;
use aes::cipher::{BlockDecrypt, KeyInit, generic_array::GenericArray};
use base64::{Engine as _, engine::general_purpose};

/// AES decrypt for bykc api, use Base64, ECB mode, PKCS5Padding
pub fn aes_decrypt(data: &str, key: &str) -> String {
    // 将 Base64 编码的加密数据解码为字节数组
    let encrypted_bytes = general_purpose::STANDARD.decode(data).unwrap();
    // 将密钥转换为字节数组
    let key_bytes = key.as_bytes();
    // 创建 AES 解密器
    let cipher = Aes128::new_from_slice(key_bytes).unwrap();
    // 创建输出缓冲区
    let mut output = vec![0u8; encrypted_bytes.len()];
    // 逐块解密数据
    for (i, chunk) in encrypted_bytes.chunks(16).enumerate() {
        let mut block = *GenericArray::from_slice(chunk);
        cipher.decrypt_block(&mut block);
        output[i * 16..(i + 1) * 16].copy_from_slice(&block);
    }
    // 移除填充
    let padding_len = output.last().map(|&x| x as usize).unwrap_or(0);
    if padding_len <= output.len() {
        output.truncate(output.len() - padding_len);
    }
    // 将解密后的数据转换为字符串
    String::from_utf8(output).unwrap()
}

#[test]
fn test_aes() {
    let env =  crate::utils::env();
    let raw= env.get("AES_RAW").unwrap();
    let key = "B55Ya5Y7FRa4CJm3";
    let decrypted = aes_decrypt(&raw, key);
    println!("{}", decrypted);
}