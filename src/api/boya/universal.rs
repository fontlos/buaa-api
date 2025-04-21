use num_bigint::BigUint;
use rand::Rng;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;

use crate::{Error, crypto, utils};

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
    /// Locate the following sections in the `app.js` with breakpoint debugging.
    ///
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
        // 考虑到这里只使用一个 RSA 公钥, 直接硬编码进去省略解析, 下面是 JS 逆向得到的公钥
        // -----BEGIN PUBLIC KEY-----
        // MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDlHMQ3B5GsWnCe7Nlo1YiG/YmH
        // dlOiKOST5aRm4iaqYSvhvWmwcigoyWTM+8bv2+sf6nQBRDWTY4KmNV7DBk1eDnTI
        // Qo6ENA31k5/tYCLEXgjPbEjCK9spiyB62fCT6cqOhbamJB0lcDJRO6Vo1m3dy+fD
        // 0jbxfDVBBNtyltIsDQIDAQAB
        // -----END PUBLIC KEY-----
        let rsa = crypto::rsa::RsaPublicKey::new(
            BigUint::parse_bytes("160888176901620604305352861809177261938762057016319779754807041689583497071063662739822117937549470133551626633870393770286681244760722711708602120884986220998770095524522498502415051190478765491126480852711587593365046521407362610305824222413933838389111328227344865362271410246251876261314138515871039106061".as_bytes(), 10).unwrap(),
            BigUint::parse_bytes("65537".as_bytes(), 10).unwrap(),
            );

        // 获取 token
        let config = self.config.read().unwrap();
        let token = match &config.boya_token {
            Some(t) => t,
            None => return Err(Error::APIError("No Boya Token".to_string())),
        };
        // 首先初始化 RSA, 设置公钥
        // 这是查询参数, 然后被 sha1 处理
        let sha1_query = crypto::hash::sha1(query);
        // sk参数, rsa sha1_query
        let sk = rsa.encrypt_to_string(&sha1_query.as_bytes());
        // 十六位随机字符
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::rng();
        let aes_key: String = (0..16)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        // ak参数, rsa aes_key
        let ak = rsa.encrypt_to_string(&aes_key.as_bytes());
        // 这是请求的负载, 是使用 aes 加密的查询参数
        let body = crypto::aes::aes_encrypt_ecb(query, &aes_key);
        let time = utils::get_time();

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
        let res = res.text().await?;
        let res = res.trim_matches('"');
        let res = crypto::aes::aes_decrypt(&res, &aes_key);
        let status = serde_json::from_str::<BoyaStatus>(&res)?;
        if status.status == "98005399" {
            return Err(Error::LoginExpired("Boya Login Expired".to_string()));
        }
        if status.status == "1" {
            return Err(Error::APIError(status.errmsg));
        }
        if status.status != "0" {
            return Err(Error::APIError(status.errmsg));
        }
        Ok(res)
    }
}

#[test]
fn test_rand() {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    // 创建一个随机数生成器
    let mut rng = rand::rng();

    // 生成长度为16的随机字符串
    let random_string: String = (0..16)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    println!("随机生成的字符串: {}", random_string);
}
