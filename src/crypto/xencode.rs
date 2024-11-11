use base64::alphabet::Alphabet;
use base64::engine::{Engine, GeneralPurpose, GeneralPurposeConfig};

/// 将字符串字节数组每四位转换后合并成一个新的数组
fn str2vec(a: &str) -> Vec<u32> {
    let c = a.len();
    let mut v = Vec::new();
    for i in (0..c).step_by(4) {
        let mut value: u32 = 0;
        if i < c {
            value |= a.as_bytes()[i] as u32;
        }
        if i + 1 < c {
            value |= (a.as_bytes()[i + 1] as u32) << 8;
        }
        if i + 2 < c {
            value |= (a.as_bytes()[i + 2] as u32) << 16;
        }
        if i + 3 < c {
            value |= (a.as_bytes()[i + 3] as u32) << 24;
        }
        v.push(value);
    }

    v
}

/// XEncode for gw api</br>
/// A custom encoding, the last step is Base64 encoding
pub fn x_encode(str: &str, key: &str) -> String {
    if str.is_empty() {
        return String::new();
    }

    let mut pw = str2vec(str);
    let mut pwdkey = str2vec(key);

    let n = pw.len() as u32;

    pw.push(str.len() as u32);
    if pwdkey.len() < 4 {
        pwdkey.resize(4, 0);
    }

    let mut z = str.len() as u32;
    let mut y;
    let c = 2654435769;
    let mut m;
    let mut e;
    let mut p;
    let q = (6 + 52 / (n + 1)) as u32;
    let mut d = 0u32;

    for _ in 0..q {
        d = d.wrapping_add(c);
        e = (d >> 2) & 3;
        p = 0;
        while p < n {
            y = pw[(p + 1) as usize];
            m = (z >> 5 ^ y << 2)
                .wrapping_add((y >> 3 ^ z << 4) ^ (d ^ y))
                .wrapping_add(pwdkey[(p & 3) as usize ^ e as usize] ^ z);
            pw[p as usize] = pw[p as usize].wrapping_add(m);
            z = pw[p as usize];
            p += 1;
        }
        y = pw[0];
        m = (z >> 5 ^ y << 2)
            .wrapping_add((y >> 3 ^ z << 4) ^ (d ^ y))
            .wrapping_add(pwdkey[(p & 3) as usize ^ e as usize] ^ z);
        pw[n as usize] = pw[n as usize].wrapping_add(m);
        z = pw[n as usize];
    }

    let mut bytes = Vec::new();
    for i in pw{
        bytes.push((i & 0xff) as u8);
        bytes.push((i >> 8 & 0xff) as u8);
        bytes.push((i >> 16 & 0xff) as u8);
        bytes.push((i >> 24 & 0xff) as u8);
    }
    let alphabet = Alphabet::new("LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA").unwrap();
    let engine = GeneralPurpose::new(&alphabet, GeneralPurposeConfig::new());
    format!("{{SRBX1}}{}",engine.encode(bytes))
}

#[test]
fn test_xencoder() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();
    let ip = env.get("IP").unwrap();
    let data = format!("{{\"username\":\"{username}\",\"password\":\"{password}\",\"ip\":\"{ip}\",\"acid\":\"62\",\"enc_ver\":\"srun_bx1\"}}");
    let res = x_encode(&data,"8e4e83f094924913acc6a9d5149015aafc898bd38ba8f45be6bd0f9edd450403");
    assert_eq!(
        &res,
        "{SRBX1}p00873sYXXqOdVgJGG3pnnRbF99gDX6b03gBghCUqOXfT9du5GeouZ+H/uR78LqlLg+LJm9XZet3JZYnyZGQciC5GtboAz1QQVvkx07f/pht93EBRF9fdqNYRJIiWE3KzRWQozPndYgz1GTkUpzph+=="
    );
}