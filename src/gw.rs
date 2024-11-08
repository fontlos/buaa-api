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
        // 在 Windows 平台上先检测 WiFi 名称, 不符合也默认当连接成功了
        #[cfg(target_os = "windows")]
        if &get_wifi().unwrap() != "BUAA-WiFi" {
            return Ok(())
        }

        // 获取本机 IP
        let ip = match get_ip() {
            Some(s) => s,
            None => return Err(SessionError::LoginError(String::from("Cannot get IP address")))
        };

        // 从重定向 URL 中获取 ACID
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
        let data = format!(r#"{{username: "{un}",password: "{pw}",ip: "{ip}",acid: "{ac_id}",enc_ver: "srun_bx1"}}"#);
        // 自带前缀
        let info = utils::info(&data, &token);

        // 计算加密后的密码, 并且后补前缀
        let password_md5 = utils::md5(pw, &token);

        // 计算校验和, 参数顺序如下
        // token username token password_md5 token ac_id token ip token n token type token info
        let check_str = format!("{token}{un}{token}{password_md5}{token}{ac_id}{token}{ip}{token}200{token}1{token}{info}");
        let chk_sum = utils::sha1(&check_str);

        // 构造登录 URL 并登录
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