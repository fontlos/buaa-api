//! GW API, For BUAA WiFi Login

use crate::crypto::{hash, x_encode};
use crate::{utils, Context, Error};

impl Context {
    /// # BUAA WiFi Login
    /// - Input: Username, Password
    /// This API is independent of other APIs and does not require cookies, so you need to provide a separate username and password <br>
    /// ```rust
    /// use buaa::Session;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let session = Session::new_in_memory();
    ///     session.wifi_login("username", "password").await.unwrap();
    /// }
    /// ```
    pub async fn wifi_login(&self, un: &str, pw: &str) -> crate::Result<()> {
        // 先检测 WiFi 名称, 不符合就直接返回以节省时间
        // 为了避免一些不必要的错误, 如果无法获取到 SSID 那么也尝试连接
        if let Some(s) = utils::get_wifi_ssid() {
            if s != "BUAA-WiFi" {
                return Ok(());
            }
        }

        // 获取本机 IP
        let ip = match utils::get_ip() {
            Some(s) => s,
            None => {
                return Err(Error::LoginError(String::from(
                    "Cannot get IP address",
                )))
            }
        };

        // 从重定向 URL 中获取 ACID
        // 接入点, 不知道具体作用但是关系到登录之后能否使用网络, 如果用固定值可能出现登陆成功但网络不可用
        let res = self.get("http://gw.buaa.edu.cn").send().await?;
        let url = res.url().as_str();
        let ac_id = match utils::get_value_by_lable(url, "ac_id=", "&") {
            Some(s) => s,
            None => return Err(Error::LoginError("No AC ID".to_string())),
        };

        // 获取 Challenge Token
        let time = &utils::get_time().to_string()[..];
        let params = [
            ("callback", time),
            ("username", un),
            ("ip", &ip),
            ("_", time),
        ];
        let res = self
            .get("https://gw.buaa.edu.cn/cgi-bin/get_challenge")
            .query(&params)
            .send()
            .await?;
        let token = if res.status().is_success() {
            let html = res.text().await.unwrap();
            match utils::get_value_by_lable(&html, "\"challenge\":\"", "\"") {
                Some(s) => s,
                None => return Err(Error::LoginError("No Challenge Value".to_string())),
            }
        } else {
            return Err(Error::LoginError(
                "Request Failed. Maybe wrong username and password".to_string(),
            ));
        };

        // 计算登录信息
        // 注意因为是直接格式化字符串而非通过json库转成标准json, 所以必须保证格式完全正确, 无空格, 键值对都带双引号
        let data = format!(
            r#"{{"username":"{un}","password":"{pw}","ip":"{ip}","acid":"{ac_id}","enc_ver":"srun_bx1"}}"#
        );
        // 自带前缀
        let info = x_encode(&data, &token);

        // 计算加密后的密码, 并且后补前缀
        let password_md5 = hash::md5_hmac(pw, &token);

        // 计算校验和, 参数顺序如下
        //                             token username token password_md5 token ac_id token ip token n token type token info
        let check_str = format!("{token}{un}{token}{password_md5}{token}{ac_id}{token}{ip}{token}200{token}1{token}{info}");
        let chk_sum = hash::sha1(&check_str);

        // 构造登录 URL 并登录
        // 暂时不知道后面五个参数有无修改必要
        let params = [
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
        let res = self
            .get("https://gw.buaa.edu.cn/cgi-bin/srun_portal")
            .query(&params)
            .send()
            .await?;
        let res = res.text().await?;
        // 注意没有考虑免费流量用尽或者全部流量用尽的情况
        // "ploy_msg":"您的免费30G流量已用尽，当前正在使用套餐流量。"
        if res.contains(r#""error":"ok""#) {
            return Ok(());
        } else {
            return Err(Error::LoginError(format!("Response: {res}")));
        }
    }

    pub async fn wifi_logout(&self, un: &str) -> crate::Result<()> {
        // 先检测 WiFi 名称, 不符合就直接返回以节省时间
        // 为了避免一些不必要的错误, 如果无法获取到 SSID 那么也尝试连接
        if let Some(s) = utils::get_wifi_ssid() {
            if s != "BUAA-WiFi" {
                return Ok(());
            }
        }

        // 获取本机 IP
        let ip = match utils::get_ip() {
            Some(s) => s,
            None => {
                return Err(Error::LoginError(String::from(
                    "Cannot get IP address",
                )))
            }
        };

        // 从重定向 URL 中获取 ACID
        // 接入点, 不知道具体作用但是关系到登录之后能否使用网络, 如果用固定值可能出现登陆成功但网络不可用
        let res = self.get("http://gw.buaa.edu.cn").send().await?;
        let url = res.url().as_str();
        let ac_id = match utils::get_value_by_lable(url, "ac_id=", "&") {
            Some(s) => s,
            None => return Err(Error::LoginError("No AC ID".to_string())),
        };

        let time = &utils::get_time().to_string()[..];

        // 构造登出 URL 并登录
        // 暂时不知道后面五个参数有无修改必要
        let params = [
            ("callback", time),
            ("action", "logout"),
            ("username", un),
            ("ac_id", &ac_id),
            ("ip", &ip)
        ];

        let res = self
            .get("https://gw.buaa.edu.cn/cgi-bin/srun_portal")
            .query(&params)
            .send()
            .await?;

        let res = res.text().await?;
        if res.contains(r#""error":"ok""#) {
            Ok(())
        } else {
            Err(Error::APIError(format!("WiFi logout failed. Response: {res}")))
        }
    }
}
