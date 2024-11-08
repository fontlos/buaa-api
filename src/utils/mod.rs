use std::collections::HashMap;
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
pub fn env() -> HashMap<String, String> {
    let env_str = File::open(".env").unwrap();
    let env: HashMap<String, String> = serde_json::from_reader(env_str).unwrap();
    env
}

pub fn get_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn get_value_by_lable(text: &str, right: &str, left: &str) -> Option<String> {
    if let Some(start) = text.find(right) {
        // 计算开始位置
        let value_start = start + right.len();
        // 查找结束位置
        if let Some(end) = text[value_start..].find(left) {
            // 提取值
            Some(String::from(&text[value_start..value_start + end]))
        } else {
            // 理论上不可能出错
            None
        }
    } else {
        None
    }
}

#[cfg(feature = "js")]
mod js {
    use boa_engine::{Context, Source};

    pub fn md5(data:&str, key: &str) -> String {
        const MD5:&[u8] = include_bytes!("md5.js");
        let mut context = Context::builder().build().unwrap();
        context.eval(Source::from_bytes(MD5)).unwrap();
        let input = format!(r#"md5("{}", "{}");"#, data, key);
        context.eval(Source::from_bytes(&input)).unwrap().to_string(&mut context).unwrap().to_std_string_escaped()
    }

    pub fn info(data:&str, key: &str) -> String {
        const INFO:&[u8] = include_bytes!("info.js");
        let mut context = Context::builder().build().unwrap();
        context.eval(Source::from_bytes(INFO)).unwrap();
        let input = format!(r#"info({}, "{}")"#, data, key);
        format!("{}", context.eval(Source::from_bytes(&input)).unwrap().to_string(&mut context).unwrap().to_std_string_escaped())
    }

    pub fn sha1(data:&str) -> String {
        const SHA1:&[u8] = include_bytes!("sha1.js");
        let mut context = Context::builder().build().unwrap();
        context.eval(Source::from_bytes(SHA1)).unwrap();
        let input = format!(r#"sha1("{}")"#, data);
        format!("{}", context.eval(Source::from_bytes(&input)).unwrap().to_string(&mut context).unwrap().to_std_string_escaped())
    }

    #[test]
    fn test_md5() {
        let env = super::env();
        let password = env.get("PASSWORD").unwrap();
        let res = md5(&password, "8e4e83f094924913acc6a9d5149015aafc898bd38ba8f45be6bd0f9edd450403");
        assert_eq!("1353748f3f9dd51b79cd869846c94cf4", &res);
    }

    #[test]
    fn test_info() {
        let env = super::env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();
        let ip = env.get("IP").unwrap();
        let data = format!(r#"{{username: "{username}",password: "{password}",ip: "{ip}",acid: "62",enc_ver: "srun_bx1"}}"#);
        let res = info(&data, "8e4e83f094924913acc6a9d5149015aafc898bd38ba8f45be6bd0f9edd450403");
        assert_eq!(
            &res,
            "{SRBX1}p00873sYXXqOdVgJGG3pnnRbF99gDX6b03gBghCUqOXfT9du5GeouZ+H/uR78LqlLg+LJm9XZet3JZYnyZGQciC5GtboAz1QQVvkx07f/pht93EBRF9fdqNYRJIiWE3KzRWQozPndYgz1GTkUpzph+=="
        );
    }

    #[test]
    fn test_check_sum() {
        let env = super::env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();
        let ip = env.get("IP").unwrap();
        let except_check_str = env.get("CHECK_STR").unwrap();
        let ac_id = "62";
        let token = "4ff7769a5e6715fa2eed7b4dfd0d6b0d93d16caa6adb42c12b72bbf10f862dee";
        let data = format!(r#"{{username: "{username}",password: "{password}",ip: "{ip}",acid: "{ac_id}",enc_ver: "srun_bx1"}}"#);
        let info = info(&data, &token);
        let password_md5 = md5(password, &token);
        let check_str = format!("{token}{username}{token}{password_md5}{token}{ac_id}{token}{ip}{token}200{token}1{token}{info}");
        assert_eq!(
            &check_str,
            except_check_str
        );
        let check_sum = sha1(&check_str);
        assert_eq!(
            &check_sum,
            "1de33e00f1bbc022d642956cc639e009cb639533"
        );
    }
}

#[cfg(feature = "js")]
pub use js::*;