use base64::alphabet::Alphabet;
use base64::engine::{Engine, GeneralPurpose, GeneralPurposeConfig};
use sha1::{Sha1, Digest};
use md5::Md5;
use hmac::{Hmac, Mac};

use std::net::UdpSocket;

use crate::{Session, SessionError, utils};

impl Session {
    /// # BUAA WiFi Login
    /// This API is independent of other APIs and does not require cookies, so you need to provide a separate username and password </br>
    /// And because this API uses a JavaScript engine, you need to enable `js` feature
    /// ```rust
    /// use buaa::Session;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let session = Session::new_in_memory();
    ///     session.gw_login("username", "password").await.unwrap();
    /// }
    /// ```
    pub async fn gw_login(&self, un: &str, pw: &str) -> Result<(), SessionError> {
        // 在 Windows 平台上先检测 WiFi 名称, 不符合就直接返回
        if &get_wifi().unwrap() != "BUAA-WiFi" {
            return Ok(())
        }

        // 获取本机 IP
        let ip = match get_ip() {
            Some(s) => s,
            None => return Err(SessionError::LoginError(String::from("Cannot get IP address")))
        };

        // 从重定向 URL 中获取 ACID
        // 接入点, 不知道具体作用但是关系到登录之后能否使用网络, 如果用固定值可能出现登陆成功但网络不可用
        let res = self.get("http://gw.buaa.edu.cn")
            .send()
            .await
            .unwrap();
        let url = res.url().as_str();
        let ac_id = match utils::get_value_by_lable(url, "ac_id=", "&") {
            Some(s) => s,
            None => return Err(SessionError::RequestError)
        };

        // 获取 Challenge Token
        let time = &utils::get_time().to_string()[..];
        let params= [
            ("callback", time),
            ("username", un),
            ("ip", &ip),
            ("_", time),
        ];
        let res = self.get("https://gw.buaa.edu.cn/cgi-bin/get_challenge")
            .query(&params)
            .send()
            .await
            .unwrap();
        let token = if res.status().is_success() {
            let html = res.text().await.unwrap();
            match utils::get_value_by_lable(&html, "\"challenge\":\"", "\"") {
                Some(s) => s,
                None => return Err(SessionError::RequestError)
            }
        } else {
            return Err(SessionError::RequestError);
        };

        // 计算登录信息
        // 注意因为是直接格式化字符串而非通过json库转成标准json, 所以必须保证格式完全正确, 无空格, 键值对都带双引号
        let data = format!(r#"{{"username":"{un}","password":"{pw}","ip":"{ip}","acid":"{ac_id}","enc_ver":"srun_bx1"}}"#);
        // 自带前缀
        let info = x_encode(&data, &token);

        // 计算加密后的密码, 并且后补前缀
        let mut hmac = Hmac::<Md5>::new_from_slice(token.as_bytes()).unwrap();
        hmac.update(pw.as_bytes());
        let res = hmac.finalize().into_bytes();
        let password_md5 = hex::encode(&res);

        // 计算校验和, 参数顺序如下
        //                             token username token password_md5 token ac_id token ip token n token type token info
        let check_str = format!("{token}{un}{token}{password_md5}{token}{ac_id}{token}{ip}{token}200{token}1{token}{info}");
        let hash = Sha1::digest(check_str.as_bytes());
        let chk_sum = hex::encode(&hash);

        // 构造登录 URL 并登录
        // 暂时不知道后面五个参数有无修改必要
        let params= [
            ("callback", time),
            ("action", "login"),
            ("username", un),
            ("password", &format!("{{MD5}}{password_md5}")),
            ("ac_id", &ac_id),
            ("ip", &ip),
            ("chksum", &chk_sum),
            ("info", &info),
            ("n", "200"),
            ("type", "1"),
            ("os", "Windows+10"),
            ("name", "Windows"),
            ("double_stack", "0"),
            ("_", time),
        ];
        let res = self.get("https://gw.buaa.edu.cn/cgi-bin/srun_portal")
            .query(&params)
            .send()
            .await
            .unwrap();
        let res = res.text().await.unwrap();
        if res.contains("Login is successful"){
            return Ok(())
        } else {
            return Err(SessionError::LoginError(format!("Response: {res}")))
        }
    }
}

// TODO 其他平台暂时按成功处理
#[cfg(not(target_os = "windows"))]
fn get_wifi() -> &str {
    "BUAA-WiFi"
}

// 因为没有其他测试平台所以只做了Windows
#[cfg(target_os = "windows")]
fn get_wifi() -> windows::core::Result<String> {
    use windows::{
        Win32::NetworkManagement::WiFi::{
            WlanOpenHandle, WlanEnumInterfaces, WlanQueryInterface, WlanCloseHandle, WlanFreeMemory,
            WLAN_OPCODE_VALUE_TYPE, WLAN_INTERFACE_INFO_LIST, WLAN_CONNECTION_ATTRIBUTES,
            wlan_intf_opcode_current_connection
        },
        Win32::Foundation::{HANDLE, ERROR_SUCCESS}
    };
    unsafe {
        let mut client_handle: HANDLE = HANDLE::default();
        let mut negotiated_version: u32 = 0;
        let result = WlanOpenHandle(2, Some(std::ptr::null()), &mut negotiated_version, &mut client_handle);
        if result != ERROR_SUCCESS.0 {
            return Ok(format!("WlanOpenHandle failed with error: {}", result));
        }

        let mut p_if_list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        let result = WlanEnumInterfaces(client_handle, Some(std::ptr::null()), &mut p_if_list);
        if result != ERROR_SUCCESS.0 {
            WlanCloseHandle(client_handle, Some(std::ptr::null()));
            return Ok(format!("WlanEnumInterfaces failed with error: {}", result));
        }

        let if_list = &*p_if_list;
        let mut ssid = String::from("Unknown Error");

        for i in 0..if_list.dwNumberOfItems {
            let p_if_info = &if_list.InterfaceInfo[i as usize];
            let mut p_conn_attr: *mut WLAN_CONNECTION_ATTRIBUTES = std::ptr::null_mut();
            let mut conn_attr_size: u32 = 0;
            let mut op_code: WLAN_OPCODE_VALUE_TYPE = WLAN_OPCODE_VALUE_TYPE(0);

            let result = WlanQueryInterface(
                client_handle,
                &p_if_info.InterfaceGuid,
                wlan_intf_opcode_current_connection,
                Some(std::ptr::null()),
                &mut conn_attr_size,
                &mut p_conn_attr as *mut _ as *mut _,
                Some(&mut op_code),
            );

            if result == ERROR_SUCCESS.0 && !p_conn_attr.is_null() {
                let conn_attr = &*p_conn_attr;
                ssid = String::from_utf8_lossy(&conn_attr.wlanAssociationAttributes.dot11Ssid.ucSSID[..conn_attr.wlanAssociationAttributes.dot11Ssid.uSSIDLength as usize]).to_string();
                WlanFreeMemory(p_conn_attr as *mut _);
                break;
            }
        }

        WlanFreeMemory(p_if_list as *mut _);
        WlanCloseHandle(client_handle, Some(std::ptr::null()));
        return Ok(ssid)
    }
}

fn get_ip() -> Option<String> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None
    };
    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None
    }
    match socket.local_addr() {
        Ok(a) => Some(a.ip().to_string()),
        Err(_) => None
    }
}

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

/// 一个自定义编码, 最后一步经过 Base64 编码
fn x_encode(str: &str, key: &str) -> String {
    if str.is_empty() {
        return String::new();
    }

    let mut pw = str2vec(str);
    let mut pwdkey = str2vec(key);
    pw.push(str.len() as u32);

    if pwdkey.len() < 4 {
        pwdkey.resize(4, 0);
    }

    let n = (pw.len() - 1) as u32;
    let mut z = pw[n as usize];
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

#[tokio::test]
async fn test_gw_login() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();
    let session = Session::new_in_memory();
    match session.gw_login(&username, &password).await {
        Ok(_) => (),
        Err(e) => eprintln!("{:?}", e)
    }
}

#[test]
fn test_get_wifi() {
    let s = get_wifi().unwrap();
    println!("{}",s)
}

#[test]
fn test_get_ip() {
    let s = get_ip().unwrap();
    println!("{}",s)
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