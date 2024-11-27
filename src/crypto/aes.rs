use aes::cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes128;
use base64::{engine::general_purpose, Engine as _};

/// AES encrypt, use Base64, ECB mode, PKCS5Padding
pub fn aes_encrypt_ecb(data: &str, key: &str) -> String {
    // 将密钥转换为字节数组
    let key_bytes = key.as_bytes();
    // 创建 AES 加密器
    let cipher = Aes128::new_from_slice(key_bytes).unwrap();
    // 将输入数据转换为字节数组
    let mut data_bytes = data.as_bytes().to_vec();
    // 计算填充长度
    let padding_len = 16 - (data_bytes.len() % 16);
    // 添加 PKCS5Padding
    for _ in 0..padding_len {
        data_bytes.push(padding_len as u8);
    }
    // 创建输出缓冲区
    let mut output = vec![0u8; data_bytes.len()];
    // 逐块加密数据
    for (i, chunk) in data_bytes.chunks(16).enumerate() {
        let mut block = *GenericArray::from_slice(chunk);
        cipher.encrypt_block(&mut block);
        output[i * 16..(i + 1) * 16].copy_from_slice(&block);
    }
    // 将加密后的数据进行 Base64 编码
    general_purpose::STANDARD.encode(&output)
}

/// AES encrypt, use Base64, CBC mode, ZeroPadding
pub fn aes_encrypt_cbc(data: &str, key: &str, iv: &str) -> String {
    // 将密钥和初始向量转换为字节数组
    let key_bytes = key.as_bytes();
    let iv_bytes = iv.as_bytes();

    // 创建 AES 加密器
    let cipher = Aes128::new_from_slice(key_bytes).unwrap();

    // 将输入数据转换为字节数组
    let mut data_bytes = data.as_bytes().to_vec();

    // 计算填充长度
    let padding_len = 16 - (data_bytes.len() % 16);

    // 添加 ZeroPadding
    for _ in 0..padding_len {
        data_bytes.push(0u8);
    }

    // 创建输出缓冲区
    let mut output = vec![0u8; data_bytes.len()];

    // 初始块使用初始向量进行加密
    let mut block = GenericArray::from_mut_slice(&mut output[..16]);
    block.copy_from_slice(iv_bytes);
    cipher.encrypt_block(&mut block);

    // 逐块加密数据
    for (i, chunk) in data_bytes.chunks(16).enumerate() {
        let (prev_block, current_block) = output.split_at_mut(i * 16);
        let mut block = GenericArray::from_mut_slice(&mut current_block[..16]);

        // 使用前一个块的加密结果与当前块进行异或操作
        if i == 0 {
            // 第一个块使用初始向量进行异或操作
            for j in 0..16 {
                block[j] = chunk[j] ^ iv_bytes[j];
            }
        } else {
            // 后续块使用前一个块的加密结果进行异或操作
            for j in 0..16 {
                block[j] = chunk[j] ^ prev_block[prev_block.len() - 16 + j];
            }
        }

        // 加密当前块
        cipher.encrypt_block(&mut block);
    }

    // 将加密后的数据进行 Base64 编码
    general_purpose::STANDARD.encode(&output)
}

/// AES decrypt, use Base64, ECB mode, PKCS5Padding
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
fn test_aes_encrypt_ecb() {
    let raw = "{\"pageNumber\":1,\"pageSize\":20}";
    let key = "SenQBA8xn6CQGNJs";
    let encrypted = aes_encrypt_ecb(&raw, key);
    assert_eq!("RdzgYtkdw+V1Y5t4ieLoqjLJDIll1yDnqV4R1I+E/yM=", encrypted);
}

#[test]
fn test_aes_encrypt_cbc() {
    let raw = r#"{"sqlid":"171256358365871757581efaed47d8396a4dd1336548d4","yhlx":"2"}"#;
    let key = "inco12345678ocni";
    let iv = "ocni12345678inco";
    let encrypted = aes_encrypt_cbc(&raw, key, iv);
    assert_eq!("sjMMi2wbmqqFOAChr9uGQhPMjU9aXylfswLzenO+ne0BUNGx9zPP0sbOPO3dlds6yQp7lejz7U99uiYPjfcRWjCa/peJWOEvc+MljRS4x3k=", encrypted);
}

#[test]
fn test_aes_decrypt() {
    let env = crate::utils::env();
    let raw = env.get("AES_RAW").unwrap();
    let key = "B55Ya5Y7FRa4CJm3";
    let decrypted = aes_decrypt(&raw, key);
    println!("{}", decrypted);
}
