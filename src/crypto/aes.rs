use aes::Aes128;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};
use base64::{Engine as _, engine::general_purpose};

/// AES encrypt, use Base64, ECB mode, PKCS5Padding
pub fn aes_encrypt_ecb(data: &[u8], key: &[u8]) -> String {
    // 将密钥转换为字节数组
    let key_bytes = key;
    // 创建 AES 加密器
    let cipher = Aes128::new_from_slice(key_bytes).unwrap();
    // 将输入数据转换为字节数组
    let mut data_bytes = data.to_vec();
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

/// AES decrypt, use Base64, ECB mode, PKCS5Padding
pub fn aes_decrypt_ecb(data: &str, key: &[u8]) -> String {
    // 将 Base64 编码的加密数据解码为字节数组
    let encrypted_bytes = general_purpose::STANDARD.decode(data).unwrap();
    // 创建 AES 解密器
    let cipher = Aes128::new_from_slice(key).unwrap();
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

/// AES encrypt, use Base64, CBC mode, ZeroPadding
pub fn aes_encrypt_cbc(data: &[u8], key: &[u8], iv: &[u8]) -> String {
    // 创建 AES 加密器
    let cipher = Aes128::new_from_slice(key).unwrap();

    // 将输入数据转换为字节数组
    let mut data_bytes = data.to_vec();

    // 计算填充长度
    let padding_len = 16 - (data_bytes.len() % 16);

    // 添加 ZeroPadding
    data_bytes.extend(vec![0u8; padding_len]);

    // 创建输出缓冲区
    let mut output = vec![0u8; data_bytes.len()];

    // 初始块使用初始向量进行加密
    let block = GenericArray::from_mut_slice(&mut output[..16]);
    block.copy_from_slice(iv);
    cipher.encrypt_block(block);

    // 逐块加密数据
    for (i, chunk) in data_bytes.chunks(16).enumerate() {
        let (prev_block, current_block) = output.split_at_mut(i * 16);
        let block = GenericArray::from_mut_slice(&mut current_block[..16]);

        // 使用前一个块的加密结果与当前块进行异或操作
        if i == 0 {
            // 第一个块使用初始向量进行异或操作
            for j in 0..16 {
                block[j] = chunk[j] ^ iv[j];
            }
        } else {
            // 后续块使用前一个块的加密结果进行异或操作
            for j in 0..16 {
                block[j] = chunk[j] ^ prev_block[prev_block.len() - 16 + j];
            }
        }

        // 加密当前块
        cipher.encrypt_block(block);
    }

    // 将加密后的数据进行 Base64 编码
    general_purpose::STANDARD.encode(&output)
}
