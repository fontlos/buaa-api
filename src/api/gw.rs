//! GW API, For BUAA WiFi Login

use crate::{Session, SessionError, utils};
use crate::crypto::{x_encode, hash};

impl Session {
    /// # BUAA WiFi Login
    /// This API is independent of other APIs and does not require cookies, so you need to provide a separate username and password </br>
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
        if &utils::get_wifi().unwrap() != "BUAA-WiFi" {
            return Ok(())
        }

        // 获取本机 IP
        let ip = match utils::get_ip() {
            Some(s) => s,
            None => return Err(SessionError::LoginError(String::from("Cannot get IP address")))
        };

        // 从重定向 URL 中获取 ACID
        // 接入点, 不知道具体作用但是关系到登录之后能否使用网络, 如果用固定值可能出现登陆成功但网络不可用
        let res = self.get("http://gw.buaa.edu.cn")
            .send()
            .await?;
        let url = res.url().as_str();
        let ac_id = match utils::get_value_by_lable(url, "ac_id=", "&") {
            Some(s) => s,
            None => return Err(SessionError::NoToken(String::from("ac_id"))),
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
            .await?;
        let token = if res.status().is_success() {
            let html = res.text().await.unwrap();
            match utils::get_value_by_lable(&html, "\"challenge\":\"", "\"") {
                Some(s) => s,
                None => return Err(SessionError::NoToken(String::from("gw_login"))),
            }
        } else {
            return Err(SessionError::NoToken(String::from("gw_login")));
        };

        // 计算登录信息
        // 注意因为是直接格式化字符串而非通过json库转成标准json, 所以必须保证格式完全正确, 无空格, 键值对都带双引号
        let data = format!(r#"{{"username":"{un}","password":"{pw}","ip":"{ip}","acid":"{ac_id}","enc_ver":"srun_bx1"}}"#);
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
            .await?;
        let res = res.text().await?;
        if res.contains("Login is successful"){
            return Ok(())
        } else {
            return Err(SessionError::LoginError(format!("Response: {res}")))
        }
    }
}
