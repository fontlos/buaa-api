use std::error::Error as StdError;

use crate::crypto;
use crate::error::Error;
use crate::utils;

use super::info;

static CHECK: &[u8] = b"\"error\":\"ok\"";

impl super::WifiApi {
    /// # BUAA WiFi Login
    ///
    /// This API is independent of other APIs and does not require cookies,
    /// so you need to provide a separate username and password
    ///
    /// ## Example
    ///
    /// ```rust
    /// use buaa::Context;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let context = Context::new();
    ///     context.set_account("username", "password");
    ///     let wifi = context.wifi();
    ///     wifi.login().await.unwrap();
    /// }
    /// ```
    pub async fn login(&self) -> crate::Result<()> {
        let cred = self.cred.load();
        let un = cred.username()?;
        let pw = cred.password()?;
        // 先检测 WiFi 名称, 不符合就直接返回以节省时间
        // 但是手机上不知道怎么获取, 所以如果无法获取到 SSID 那么也尝试连接
        if let Some(s) = info::ssid() {
            if s != "BUAA-WiFi" {
                return Ok(());
            }
        }

        // 获取本机 IP
        let ip = match info::ip() {
            Some(s) => s,
            None => return Err(Error::Network(String::from("Cannot get IP address"))),
        };

        // 从重定向 URL 中获取 ACID 接入点
        // 不知道具体作用但是关系到登录之后能否使用网络, 如果用固定值可能出现登陆成功但网络不可用
        // 这里检查一下有无 DNS 错误, 如果有那证明我们没有连接到目标网络
        let res = match self.client.get("http://gw.buaa.edu.cn").send().await {
            Ok(res) => res,
            Err(e) => {
                let err = e
                    .source()
                    .and_then(|e1| e1.source())
                    .map(|e2| e2.to_string())
                    .unwrap_or_else(|| e.to_string());

                if err == "dns error" {
                    return Err(Error::Network("Not connect to BUAA-WiFi".to_string()));
                } else {
                    return Err(Error::Network(err));
                }
            }
        };

        let url = res.url().as_str().as_bytes();
        let ac_id = match utils::parse_by_tag(url, "ac_id=", "&") {
            Some(s) => s,
            None => return Err(Error::server("[Wifi] No AC ID")),
        };

        // 获取 Challenge Token
        let time = utils::get_time_millis().to_string();
        let time = time.as_str();
        let params = [
            ("callback", time),
            ("username", un),
            ("ip", &ip),
            ("_", time),
        ];
        let res = self
            .client
            .get("https://gw.buaa.edu.cn/cgi-bin/get_challenge")
            .query(&params)
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(Error::server("[Wifi] Request for challenge value failed"));
        };
        let bytes = res.bytes().await.unwrap();
        let token = match utils::parse_by_tag(&bytes, "\"challenge\":\"", "\"") {
            Some(s) => s,
            None => return Err(Error::server("[Wifi] No challenge value")),
        };
        let token_bytes = token.as_bytes();

        // 计算登录信息
        let info = serde_json::json!({
            "username": un,
            "password": pw,
            "ip": ip,
            "acid": ac_id,
            "enc_ver": "srun_bx1"
        });
        let info = serde_json::to_vec(&info)?;
        // x_encode 自带前缀 {SRBX1}
        let info = crypto::xencode::x_encode(&info, token_bytes);

        // 计算加密后的密码, 并且后补前缀
        let hmac_cipher = crypto::md5::HmacMd5::new(token_bytes);
        let pw = hmac_cipher.compute(pw.as_bytes());
        let pw = crypto::bytes2hex(&pw);

        // 计算校验和, 参数顺序如下, 剩下的两个是 n 和 type, 固定为 200 和 1
        let sum = format!(
            "{token}{un}{token}{pw}{token}{ac_id}{token}{ip}{token}200{token}1{token}{info}"
        );
        let sum = crypto::sha1::Sha1::digest(sum.as_bytes());
        let sum = crypto::bytes2hex(&sum);

        // 构造登录 URL 并登录
        // 暂时不知道后面五个参数有无修改必要
        let params = [
            ("callback", time),
            ("action", "login"),
            ("username", un),
            ("password", &format!("{{MD5}}{pw}")),
            ("ac_id", ac_id),
            ("ip", &ip),
            ("chksum", &sum),
            ("info", &info),
            ("n", "200"),
            ("type", "1"),
            ("os", "Windows+10"),
            ("name", "Windows"),
            ("double_stack", "0"),
            ("_", time),
        ];
        let res = self
            .client
            .get("https://gw.buaa.edu.cn/cgi-bin/srun_portal")
            .query(&params)
            .send()
            .await?;
        let res = res.bytes().await?;
        // 注意没有考虑免费流量用尽或者全部流量用尽的情况
        // "ploy_msg":"您的免费30G流量已用尽，当前正在使用套餐流量。"
        if res.windows(CHECK.len()).any(|window| window == CHECK) {
            Ok(())
        } else {
            Err(Error::server(format!(
                "[Wifi] Login failed. Response: {}",
                String::from_utf8_lossy(&res)
            )))
        }
    }

    /// # BUAA WiFi Logout
    ///
    /// This API is independent of other APIs and does not require cookies,
    /// so you need to provide a separate username
    ///
    /// ## Example
    ///
    /// ```rust
    /// use buaa::Context;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let context = Context::new();
    ///     context.set_username("username");
    ///     let wifi = context.wifi();
    ///     wifi.logout().await.unwrap();
    /// }
    /// ```
    pub async fn logout(&self) -> crate::Result<()> {
        let cred = self.cred.load();
        let un = cred.username()?;
        // 先检测 WiFi 名称, 不符合就直接返回以节省时间
        // 为了避免一些不必要的错误, 如果无法获取到 SSID 那么也尝试连接
        if let Some(s) = info::ssid() {
            if s != "BUAA-WiFi" {
                return Ok(());
            }
        }

        // 获取本机 IP
        let ip = match info::ip() {
            Some(s) => s,
            None => return Err(Error::Network(String::from("Cannot get IP address"))),
        };

        // 从重定向 URL 中获取 ACID 接入点
        // 不知道具体作用但是关系到登录之后能否使用网络, 如果用固定值可能出现登陆成功但网络不可用
        // 这里检查一下有无 DNS 错误, 如果有那证明我们没有连接到目标网络
        let res = match self.client.get("http://gw.buaa.edu.cn").send().await {
            Ok(res) => res,
            Err(e) => {
                let err = e
                    .source()
                    .and_then(|e1| e1.source())
                    .map(|e2| e2.to_string())
                    .unwrap_or_else(|| e.to_string());

                if err == "dns error" {
                    return Err(Error::Network("Not connect to BUAA-WiFi".to_string()));
                } else {
                    return Err(Error::Network(err));
                }
            }
        };

        let url = res.url().as_str().as_bytes();
        let ac_id = match utils::parse_by_tag(url, "ac_id=", "&") {
            Some(s) => s,
            None => return Err(Error::server("[Wifi] No AC ID")),
        };

        let time = utils::get_time_millis().to_string();
        let time = time.as_str();

        // 构造登出 URL 并登录
        // 暂时不知道后面五个参数有无修改必要
        let params = [
            ("callback", time),
            ("action", "logout"),
            ("username", un),
            ("ac_id", ac_id),
            ("ip", &ip),
        ];

        let res = self
            .client
            .get("https://gw.buaa.edu.cn/cgi-bin/srun_portal")
            .query(&params)
            .send()
            .await?;

        let res = res.bytes().await?;
        if res.windows(CHECK.len()).any(|window| window == CHECK) {
            Ok(())
        } else {
            Err(Error::server(format!(
                "[WiFi] Logout failed. Response: {}",
                String::from_utf8_lossy(&res)
            )))
        }
    }
}
