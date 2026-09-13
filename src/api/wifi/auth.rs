use crate::crypto;
use crate::error::{Code, Error};
use crate::utils;
use crate::utils::net;
use crate::utils::time::DateTime;

// use super::info;

static CHECK: &[u8] = b"\"error\":\"ok\"";

impl super::WifiApi {
    /// # Login to BUAA WiFi
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

        // 检测网络环境, 不符合就直接返回以节省时间
        if !net::is_campus_network_reachable() {
            return Err(Error::network("Not connected to BUAA-WiFi or BUAA-Mobile")
                .with_code(Code::NetworkNotCampus));
        }

        // 获取本机 IP
        let ip = net::ip()
            .ok_or_else(|| Error::network("No local IP").with_code(Code::NetworkNoLocalIp))?;

        // 2025.04.11 更新
        // 这里会重定向到 index_1.html, 最终 url 在它的 html meta 标签里
        // 从那里获取接入点 AC ID
        // 2026.05.19 更新
        // 现在我们不再需要检测 DNS 错误, 因为直接检测网关更稳定
        let bytes = self
            .client
            .get("http://gw.buaa.edu.cn")
            .send()
            .await?
            .bytes()
            .await?;

        let ac_id = utils::parse_by_tag(&bytes, "ac_id=", "&")
            .ok_or_else(|| Error::server("No AC ID").with_label("Wifi"))?;

        // 获取 Challenge Token
        let time = DateTime::millis().to_string();
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
            return Err(Error::server("Request for challenge value failed").with_label("Wifi"));
        };
        let bytes = res.bytes().await?;
        let token = match utils::parse_by_tag(&bytes, "\"challenge\":\"", "\"") {
            Some(s) => s,
            None => return Err(Error::server("No challenge value").with_label("Wifi")),
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
                "Login failed. Response: {}",
                String::from_utf8_lossy(&res)
            ))
            .with_label("Wifi"))
        }
    }

    /// # Logout from BUAA WiFi
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

        // 检测网络环境, 不符合就直接返回以节省时间
        if !net::is_campus_network_reachable() {
            return Err(Error::network("Not connected to BUAA-WiFi or BUAA-Mobile")
                .with_code(Code::NetworkNotCampus));
        }

        // 获取本机 IP
        let ip = net::ip()
            .ok_or_else(|| Error::network("No local IP").with_code(Code::NetworkNoLocalIp))?;

        // 2025.04.11 更新
        // 这里会重定向到 index_1.html, 最终 url 在它的 html meta 标签里
        // 从那里获取接入点 AC ID
        // 2026.05.19 更新
        // 现在我们不再需要检测 DNS 错误, 因为直接检测网关更稳定
        let bytes = self
            .client
            .get("http://gw.buaa.edu.cn")
            .send()
            .await?
            .bytes()
            .await?;

        let ac_id = utils::parse_by_tag(&bytes, "ac_id=", "&")
            .ok_or_else(|| Error::server("No AC ID").with_label("Wifi"))?;

        let time = DateTime::millis().to_string();
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
                "Logout failed. Response: {}",
                String::from_utf8_lossy(&res)
            ))
            .with_label("Wifi"))
        }
    }
}
