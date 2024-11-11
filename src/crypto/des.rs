use des::Des;
use des::cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};
use hex::ToHex;


/// DES encrypt for iclass api, use ECB mode
pub fn des_encrypt(data: &str) -> String {
    // 密钥来自对 JS 的逆向分析
    let key = b"Jyd#351*";
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
    output.encode_hex()
}

#[test]
fn test_des() {
    let data = "https://iclass.buaa.edu.cn:8346/?loginName=18993F6FB7040240CF299C45D4C0468A";
    let encrypted = des_encrypt(data);
    assert_eq!(
        &encrypted,
        "d537020cd453a15ebbffa0be36acca5884015c4080bc2a5a275535579bc762354bdc69f8f17ee785e0c0996e915c3f3ea32b27c24246612d04496dfb291ec4d5825fa1b89b4d45c6dffc650b31ae2338"
    );
}
