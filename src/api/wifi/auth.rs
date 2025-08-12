use std::error::Error as StdError;

use crate::{
    crypto,
    error::{AuthError, Error},
    utils,
};

use super::utils::{get_wifi_ip, get_wifi_ssid};

static CHECK: &[u8] = b"\"error\":\"ok\"";

impl super::WifiApi {
    /// # BUAA WiFi Login
    /// This API is independent of other APIs and does not require cookies, so you need to provide a separate username and password <br>
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
        let un = match cred.username.as_ref() {
            Some(s) => s,
            None => return Err(Error::Auth(AuthError::NoUsername)),
        };
        let pw = match cred.password.as_ref() {
            Some(s) => s,
            None => return Err(Error::Auth(AuthError::NoPassword)),
        };
        // 先检测 WiFi 名称, 不符合就直接返回以节省时间
        // 但是手机上不知道怎么获取, 所以如果无法获取到 SSID 那么也尝试连接
        if let Some(s) = get_wifi_ssid() {
            if s != "BUAA-WiFi" {
                return Ok(());
            }
        }

        // 获取本机 IP
        let ip = match get_wifi_ip() {
            Some(s) => s,
            None => return Err(Error::Network(String::from("Cannot get IP address"))),
        };

        // 从重定向 URL 中获取 ACID 接入点
        // 不知道具体作用但是关系到登录之后能否使用网络, 如果用固定值可能出现登陆成功但网络不可用
        // 这里检查一下有无 DNS 错误, 如果有那证明我们没有链接到目标网络
        let res = match self.get("http://gw.buaa.edu.cn").send().await {
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

        let url = res.url().as_str();
        let ac_id = match utils::get_value_by_lable(url, "ac_id=", "&") {
            Some(s) => s,
            None => return Err(Error::Server("No AC ID".to_string())),
        };

        // 获取 Challenge Token
        let time = &utils::get_time_millis().to_string()[..];
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
        if !res.status().is_success() {
            return Err(Error::Server(
                "Request Failed. Maybe wrong username and password".to_string(),
            ));
        };
        let html = res.text().await.unwrap();
        let token = match utils::get_value_by_lable(&html, "\"challenge\":\"", "\"") {
            Some(s) => s,
            None => return Err(Error::Server("No Challenge Value".to_string())),
        };
        let token_bytes = token.as_bytes();

        // 计算登录信息
        // 注意因为是直接格式化字符串而非通过json库转成标准json, 所以必须保证格式完全正确, 无空格, 键值对都带双引号
        let data = format!(
            r#"{{"username":"{un}","password":"{pw}","ip":"{ip}","acid":"{ac_id}","enc_ver":"srun_bx1"}}"#
        );
        // 自带前缀
        let info = crypto::xencode::x_encode(data.as_bytes(), token_bytes);

        // 计算加密后的密码, 并且后补前缀
        let password_md5 = crypto::md5::md5_hmac(pw.as_bytes(), token_bytes);

        // 计算校验和, 参数顺序如下, 剩下的两个是 n 和 type, 固定为 200 和 1
        let check_str = format!(
            "{token}{un}{token}{password_md5}{token}{ac_id}{token}{ip}{token}200{token}1{token}{info}"
        );
        let chk_sum = crypto::sha1::sha1(check_str.as_bytes());

        // 构造登录 URL 并登录
        // 暂时不知道后面五个参数有无修改必要
        let params = [
            ("callback", time),
            ("action", "login"),
            ("username", un),
            ("password", &format!("{{MD5}}{password_md5}")),
            ("ac_id", ac_id),
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
        let res = res.bytes().await?;
        // 注意没有考虑免费流量用尽或者全部流量用尽的情况
        // "ploy_msg":"您的免费30G流量已用尽，当前正在使用套餐流量。"
        if res.windows(CHECK.len()).any(|window| window == CHECK) {
            Ok(())
        } else {
            Err(Error::Server(format!("Response: {}", String::from_utf8_lossy(&res))))
        }
    }

    /// # BUAA WiFi Logout
    /// This API is independent of other APIs and does not require cookies, so you need to provide a separate username <br>
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
        let un = match cred.username.as_ref() {
            Some(s) => s,
            None => return Err(Error::Auth(AuthError::NoUsername)),
        };
        // 先检测 WiFi 名称, 不符合就直接返回以节省时间
        // 为了避免一些不必要的错误, 如果无法获取到 SSID 那么也尝试连接
        if let Some(s) = get_wifi_ssid() {
            if s != "BUAA-WiFi" {
                return Ok(());
            }
        }

        // 获取本机 IP
        let ip = match get_wifi_ip() {
            Some(s) => s,
            None => return Err(Error::Network(String::from("Cannot get IP address"))),
        };

        // 从重定向 URL 中获取 ACID 接入点
        // 不知道具体作用但是关系到登录之后能否使用网络, 如果用固定值可能出现登陆成功但网络不可用
        // 这里检查一下有无 DNS 错误, 如果有那证明我们没有链接到目标网络
        let res = match self.get("http://gw.buaa.edu.cn").send().await {
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

        let url = res.url().as_str();
        let ac_id = match utils::get_value_by_lable(url, "ac_id=", "&") {
            Some(s) => s,
            None => return Err(Error::Server("No AC ID".to_string())),
        };

        let time = &utils::get_time_millis().to_string()[..];

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
            .get("https://gw.buaa.edu.cn/cgi-bin/srun_portal")
            .query(&params)
            .send()
            .await?;

        let res = res.bytes().await?;
        if res.windows(CHECK.len()).any(|window| window == CHECK) {
            Ok(())
        } else {
            Err(Error::Server(format!(
                "WiFi logout failed. Response: {}",
                String::from_utf8_lossy(&res)
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[ignore]
    #[tokio::test]
    async fn test_wifi_login() {
        let context = Context::with_auth("./data");

        let wifi = context.wifi();
        wifi.login().await.unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_wifi_logout() {
        let context = Context::with_auth("./data");

        let wifi = context.wifi();
        wifi.logout().await.unwrap();
    }
}
