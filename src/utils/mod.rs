mod get_wifi;
pub use get_wifi::get_wifi;

use std::collections::HashMap;
use std::fs::File;
use std::net::UdpSocket;
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

pub fn get_primitive_time(){
    // 获取当前的 UTC 时间
    let now_utc = time::OffsetDateTime::now_utc();

    // 获取本地时区偏移量
    let local_offset = time::UtcOffset::from_hms(8, 0, 0).unwrap();

    // 将 UTC 时间转换为本地时间
    let now_local = now_utc.to_offset(local_offset);

    // 将 OffsetDateTime 转换为 PrimitiveDateTime
    let now = time::PrimitiveDateTime::new(now_local.date(), now_local.time());
    println!("{}", now);
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

pub fn get_ip() -> Option<String> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None,
    };
    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None,
    }
    match socket.local_addr() {
        Ok(a) => Some(a.ip().to_string()),
        Err(_) => None,
    }
}

#[cfg(feature = "table")]
use tabled::{
    settings::{Alignment, Style},
    Table, Tabled,
};
#[cfg(feature = "table")]
pub fn table<T: Tabled>(tabled: Vec<T>) -> String {
    Table::new(&tabled)
        .with(Style::modern_rounded())
        .with(Alignment::center())
        .with(Alignment::center_vertical())
        .to_string()
}
