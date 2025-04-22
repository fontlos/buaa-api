#[cfg(test)]
mod tests {
    use crate::crypto::*;

    #[test]
    fn test_aes_encrypt_ecb() {
        let raw = "{\"pageNumber\":1,\"pageSize\":20}";
        let key = "SenQBA8xn6CQGNJs";
        let encrypted = aes::aes_encrypt_ecb(&raw, key);
        assert_eq!("RdzgYtkdw+V1Y5t4ieLoqjLJDIll1yDnqV4R1I+E/yM=", encrypted);
    }

    #[test]
    fn test_aes_encrypt_cbc() {
        let raw = r#"{"sqlid":"171256358365871757581efaed47d8396a4dd1336548d4","yhlx":"2"}"#;
        let key = "inco12345678ocni";
        let iv = "ocni12345678inco";
        let encrypted = aes::aes_encrypt_cbc(&raw, key, iv);
        assert_eq!(
            "sjMMi2wbmqqFOAChr9uGQhPMjU9aXylfswLzenO+ne0BUNGx9zPP0sbOPO3dlds6yQp7lejz7U99uiYPjfcRWjCa/peJWOEvc+MljRS4x3k=",
            encrypted
        );
    }

    #[test]
    fn test_aes_decrypt() {
        let env = crate::utils::env();
        let raw = env.get("AES_RAW").unwrap();
        let key = "B55Ya5Y7FRa4CJm3";
        let decrypted = aes::aes_decrypt(&raw, key);
        println!("{}", decrypted);
    }

    #[test]
    fn test_des() {
        let data = "https://iclass.buaa.edu.cn:8346/?loginName=18993F6FB7040240CF299C45D4C0468A";
        let encrypted = des::des_encrypt(data, crate::consts::CLASS_DES_KEY);
        assert_eq!(
            &encrypted,
            "d537020cd453a15ebbffa0be36acca5884015c4080bc2a5a275535579bc762354bdc69f8f17ee785e0c0996e915c3f3ea32b27c24246612d04496dfb291ec4d5825fa1b89b4d45c6dffc650b31ae2338"
        );
    }

    #[test]
    fn test_md5() {
        let data = std::fs::read("License").unwrap();
        let md5 = md5::md5(&data);
        assert_eq!(&md5, "2817feea7bcabab5909f75866950e0d3");
    }

    #[test]
    fn test_md5_hmac() {
        let data = b"HelloWorld";
        let key = "Key";
        let hmac = md5::md5_hmac(data, key);
        assert_eq!(&hmac, "219e14bef981f117479a7695dacb10c7");
    }

    #[test]
    fn test_sha1() {
        let data = b"HelloWorld";
        let sha1 = sha1::sha1(data);
        assert_eq!(&sha1, "db8ac1c259eb89d4a131b253bacfca5f319d54f2");
    }

    #[test]
    fn test_xencoder() {
        let env = crate::utils::env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();
        let ip = env.get("IP").unwrap();
        let data = format!(
            "{{\"username\":\"{username}\",\"password\":\"{password}\",\"ip\":\"{ip}\",\"acid\":\"62\",\"enc_ver\":\"srun_bx1\"}}"
        );
        let res = xencode::x_encode(
            &data,
            "8e4e83f094924913acc6a9d5149015aafc898bd38ba8f45be6bd0f9edd450403",
        );
        assert_eq!(
            &res,
            "{SRBX1}p00873sYXXqOdVgJGG3pnnRbF99gDX6b03gBghCUqOXfT9du5GeouZ+H/uR78LqlLg+LJm9XZet3JZYnyZGQciC5GtboAz1QQVvkx07f/pht93EBRF9fdqNYRJIiWE3KzRWQozPndYgz1GTkUpzph+=="
        );
    }
}
