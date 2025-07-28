use rand::Rng;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;

use crate::{Error, crypto, error::Location, utils};

#[derive(Deserialize)]
struct BoyaStatus {
    status: String,
    errmsg: String,
}

impl super::BoyaAPI {
    /// # Boya Universal Request API
    /// - Input:
    ///     - Query: JSON String for request
    ///     - URL: API URL
    ///
    /// Other Boya APIs don't need to be implemented, if you need to, you can extend it with this universal request API. <br>
    /// You can find JS code like the following by intercepting all XHR requests in the browser, via stack calls. <br>
    /// Locate the following sections in the `app.js`(Windows UA)/`main.js`(Android UA) by search `setPublicKey` and set breakpoint to debug.
    /// # JS Code
    ///  ```js
    /// var y = new h.default;
    /// y.setPublicKey(b);
    /// var x = c || {}
    ///   , w = JSON.stringify(x)
    ///   , k = (0,
    /// o.default)(w).toString()
    ///   , A = y.encrypt(k)
    ///   , _ = s.getRandomStr(16)
    ///   , S = y.encrypt(_)
    ///   , D = d.default.parse(_)
    ///   , E = l.default.encrypt(d.default.parse(w), D, {
    ///     iv: D,
    ///     mode: u.default,
    ///     padding: f.default
    /// }).toString()
    ///   , I = (new Date).getTime() + "";
    /// g.sk = A,
    /// g.ak = S,
    /// g.ts = I;
    /// var C = function(e) {
    ///     var t = d.default.parse(_)
    ///       , n = l.default.decrypt(e.data, t, {
    ///         iv: t,
    ///         mode: u.default,
    ///         padding: f.default
    ///     })
    ///       , i = d.default.stringify(n);
    ///     return i && (e.data = JSON.parse(i)),
    ///     e
    /// }
    /// ```
    ///
    /// You can find `Query` in `w = JSON.stringify(x)`
    ///
    /// # Example
    ///
    /// `getUserProfile` API
    /// - URL: `https://bykc.buaa.edu.cn/sscv/getUserProfile`
    /// - Query: `{}`
    pub async fn universal_request(&self, query: &str, url: &str) -> crate::Result<String> {
        let cred = &self.cred.load().boya_token;
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && cred.is_expired() {
            self.login().await?;
        }
        // 首先尝试获取 token, 如果没有就可以直接返回了
        let token = match cred.value() {
            Some(t) => t,
            None => return Err(Error::APIError("No Boya Token".to_string())),
        };

        // 初始化 RSA, 设置公钥
        let rsa = crypto::rsa::RsaPkcs1v15::from_pem(crate::consts::BOYA_RSA_KEY);

        // 这是查询参数, 然后被 sha1 处理
        let sha1_query = crypto::sha1::sha1(query.as_bytes());
        // sk 参数, rsa sha1_query
        let sk = rsa.encrypt_to_string(sha1_query.as_bytes());

        // AES Key, 使用十六位随机字符
        let aes_key = gen_rand_str(16);
        let aes_key = aes_key.as_bytes();
        // ak 参数, rsa aes_key
        let ak = rsa.encrypt_to_string(aes_key);

        // 请求的负载, 是使用 AES 加密的查询参数
        let body = crypto::aes::aes_encrypt_ecb(query.as_bytes(), aes_key);
        let time = utils::get_time_millis();

        // 生成请求头
        let mut header = HeaderMap::new();
        header.insert(
            HeaderName::from_bytes(b"Ak").unwrap(),
            HeaderValue::from_str(&ak).unwrap(),
        );
        header.insert(
            HeaderName::from_bytes(b"Auth_token").unwrap(),
            HeaderValue::from_str(token).unwrap(),
        );
        header.insert(
            HeaderName::from_bytes(b"Authtoken").unwrap(),
            HeaderValue::from_str(token).unwrap(),
        );
        header.insert(
            HeaderName::from_bytes(b"Sk").unwrap(),
            HeaderValue::from_str(&sk).unwrap(),
        );
        header.insert(
            HeaderName::from_bytes(b"Ts").unwrap(),
            HeaderValue::from_str(&time.to_string()).unwrap(),
        );

        // 获取 JSESSIONID
        let res = self.post(url).headers(header).json(&body).send().await?;

        // 响应体被 AES 加密了, 并且两端有引号需要去掉
        let res = res.text().await?;
        let res = res.trim_matches('"');
        let res = crypto::aes::aes_decrypt_ecb(res, aes_key);

        // 检查状态
        let status = serde_json::from_str::<BoyaStatus>(&res)?;
        if status.status == "98005399" {
            // 刷新登录 Token 的操作无需在这里执行, 如果上面刷新了, 这里还能报这个状态码那应该不是 Token 的问题
            return Err(Error::LoginExpired(Location::BOYA));
        }
        if status.status == "1" {
            return Err(Error::APIError(status.errmsg));
        }
        if status.status != "0" {
            return Err(Error::APIError(status.errmsg));
        }

        // 刷新 Token 时效
        self.cred.update(|c| {
            c.boya_token.refresh(600);
        });

        Ok(res)
    }
}

fn gen_rand_str(size: u8) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..size)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
