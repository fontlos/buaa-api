//! Boya Course API

pub mod query_course;
pub mod query_selected;
pub mod query_statistic;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;

use crate::{crypto, utils, Session, SessionError};

#[derive(Deserialize)]
struct BoyaStatus {
    status: String,
    errmsg: String,
}

impl Session {
    /// # Boya Login
    /// - Need: [`sso_login`](#method.sso_login)
    /// - Output: Token for Boya API
    pub async fn boya_login(&self) -> Result<String, SessionError> {
        let url = "https://sso.buaa.edu.cn/login?noAutoRedirect=true&service=https%3A%2F%2Fbykc.buaa.edu.cn%2Fsscv%2Fcas%2Flogin";
        // 获取 JSESSIONID
        let res = self.get(url).send().await?;
        // 未转跳就证明登录过期
        if res.url().as_str() == url {
            return Err(SessionError::LoginExpired("SSO Expired".to_string()));
        }
        let mut query = res.url().query_pairs();
        let token = match query.next() {
            Some(t) => t,
            None => return Err(SessionError::LoginError("No Token".to_string())),
        };
        if token.0 == "token" {
            return Ok(token.1.to_string());
        } else {
            return Err(SessionError::LoginError("No Token".to_string()));
        }
    }

    /// # Boya Universal Request API
    /// - Need: [`bykc_login`](#method.bykc_login)
    /// - Input:
    ///     - Query: JSON String for request
    ///     - URL: API URL
    ///     - Token from [`bykc_login`](#method.bykc_login)
    ///
    /// Other Boyaa APIs don't need to be implemented, if you need to, you can extend it with this generic request API, you can find JS code like the following by intercepting all XHR requests in the browser, via stack calls <br>
    /// Locate the following sections in the `app.js` with breakpoint debugging
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
    pub async fn boya_universal_request(
        &self,
        query: &str,
        url: &str,
        token: &str,
    ) -> Result<String, SessionError> {
        // 首先初始化 RSA, 设置公钥
        // 这是查询参数, 然后被 sha1 处理
        let sha1_query = crypto::hash::sha1(query);
        // sk参数, rsa sha1_query
        let sk = crypto::rsa(&sha1_query);
        // TODO 十六位随机字符, 这里先用固定的
        let aes_key = "SenQBA8xn6CQGNJs";
        // ak参数, rsa aes_key
        let ak = crypto::rsa(aes_key);
        // 这是请求的负载, 是使用 aes 加密的查询参数
        let body = crypto::aes::aes_encrypt_ecb(query, aes_key);
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
            return Err(SessionError::LoginExpired("Boya Login Expired".to_string()));
        }
        if status.status == "1" {
            return Err(SessionError::APIError(status.errmsg));
        }
        if status.status != "0" {
            return Err(SessionError::APIError(status.errmsg));
        }
        Ok(res)
    }

    /// # Select Course
    /// - Need: [`boya_login`](#method.boya_login)
    /// - Input:
    ///     - Token from [`boya_login`](#method.boya_login)
    ///     - Course ID from [`boya_query_course`](#method.boya_query_course)
    /// - Output: Status of the request, like `{"status":"0","errmsg":"请求成功","token":null,"data":{"courseCurrentCount":340}}`
    pub async fn boya_select_course(&self, token: &str, id: u32) -> Result<String, SessionError> {
        let query = format!("{{\"courseId\":{}}}", id);
        let url = "https://bykc.buaa.edu.cn/sscv/choseCourse";
        let res = self.boya_universal_request(&query, url, token).await?;
        Ok(res)
    }

    /// # Drop Course
    /// - Need: [`boya_login`](#method.boya_login)
    /// - Input:
    ///     - Token from [`boya_login`](#method.boya_login)
    ///     - Course ID from [`boya_query_course`](#method.boya_query_course)
    /// - Output: Status of the request, like `{"status":"0","errmsg":"请求成功","token":null,"data":{"courseCurrentCount":340}}`
    pub async fn boya_drop_course(&self, token: &str, id: u32) -> Result<String, SessionError> {
        let query = format!("{{\"id\":{}}}", id);
        let url = "https://bykc.buaa.edu.cn/sscv/delChosenCourse";
        let res = self.boya_universal_request(&query, url, token).await?;
        Ok(res)
    }
}

#[tokio::test]
async fn test_boya_select() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();

    let token = session.boya_login().await.unwrap();
    let res = session.boya_select_course(&token, 6637).await.unwrap();
    println!("{}", res);

    session.save();
}

#[tokio::test]
async fn test_boya_drop() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();

    let token = session.boya_login().await.unwrap();
    let res = session.boya_drop_course(&token, 6637).await.unwrap();
    println!("{}", res);

    session.save();
}
