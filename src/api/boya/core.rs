use serde::Serialize;

use crate::api::Location;
use crate::error::Error;
use crate::{crypto, utils};

impl super::BoyaApi {
    /// # Boya Login
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().sso.is_expired() {
            self.api::<crate::api::Sso>().login().await?;
        }

        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/cas/login
        let url = "https://sso.buaa.edu.cn/login?noAutoRedirect=true&service=https%3A%2F%2Fbykc.buaa.edu.cn%2Fsscv%2Fcas%2Flogin";
        // 获取 JSESSIONID
        let res = self.get(url).send().await?;
        // 未转跳就证明登录过期
        if res.url().as_str() == url {
            return Err(Error::auth_expired(Location::Sso));
        }
        let mut query = res.url().query_pairs();
        let token = query
            .next()
            .and_then(|t| if t.0 == "token" { Some(t.1) } else { None })
            .ok_or_else(|| Error::server("[Boya] Login failed. No token"))?;

        self.cred.set(Location::Boya, token.to_string());
        Ok(())
    }

    /// # Boya Universal Request API
    ///
    /// ## Note
    ///
    /// You should use other existing Boya APIs first.
    ///
    /// If the API you need but is not implemented, you can extend it with this universal request API.
    ///
    /// Locate the following sections in the `app.js`(Windows UA)/`main.js`(Android UA) by search `setPublicKey` and set breakpoint to debug.
    ///
    /// ```js
    /// ...
    /// y.setPublicKey(b);
    /// var x = c || {}
    ///   , w = JSON.stringify(x)
    /// ...
    /// ```
    ///
    /// You can find `Query` in `w = JSON.stringify(x)`
    ///
    /// ## Usage
    ///
    /// - Input:
    ///     - URL: API URL
    ///     - Query: JSON String for request
    /// - Output:
    ///     - JSON String for response
    ///
    /// ## Example
    ///
    /// `getUserProfile` API
    /// - URL: `https://bykc.buaa.edu.cn/sscv/getUserProfile`
    /// - Query: `{}`
    pub async fn universal_request<Q>(&self, url: &str, query: &Q) -> crate::Result<String>
    where
        Q: Serialize + ?Sized,
    {
        let cred = &self.cred.load().boya_token;
        if cred.is_expired() {
            self.login().await?;
        }
        // 首先尝试获取 token, 如果没有就可以直接返回了
        let token = match cred.value() {
            Some(t) => t,
            None => return Err(Error::auth_expired(Location::Boya)),
        };

        // 初始化 RSA, 设置公钥
        let rsa = crypto::rsa::RsaPkcs1v15::from_pem(crate::consts::BOYA_RSA_KEY);

        let query = serde_json::to_vec(query)?;
        // 这是查询参数, 然后被 sha1 处理
        // TODO: 既然需要再次加密 sha1 结果, 那也许 sha1 可以直接返回字节数组
        let sha1_query = crypto::sha1::sha1(&query);
        // sk 参数, rsa sha1_query
        let sk = rsa.encrypt_to_string(sha1_query.as_bytes());

        // AES Key, 使用十六位随机字符
        let aes_key = utils::gen_rand_str(16);
        let aes_key = aes_key.as_bytes();
        // ak 参数, rsa aes_key
        let ak = rsa.encrypt_to_string(aes_key);

        // 请求的负载, 是使用 AES 加密的查询参数
        let body = crypto::aes::aes_encrypt_ecb(&query, aes_key);
        let time = utils::get_time_millis();

        // 获取 JSESSIONID
        let res = self
            .post(url)
            .header("Ak", &ak)
            .header("Auth_token", token)
            .header("Authtoken", token)
            .header("Sk", &sk)
            .header("Ts", time.to_string())
            .json(&body)
            .send()
            .await?;

        // 响应体被 AES 加密了, 并且两端有引号需要去掉
        let res = res.bytes().await?;
        let res = &res[1..res.len() - 1];
        let res = crypto::aes::aes_decrypt_ecb(res, aes_key);

        // 检查状态
        let status = serde_json::from_str::<super::_BoyaStatus>(&res)?;
        if status.status == "98005399" {
            // 刷新登录 Token 的操作无需在这里执行, 如果上面刷新了, 这里还能报这个状态码那应该不是 Token 的问题
            return Err(Error::auth_expired(Location::Boya));
        }
        if status.status == "1" {
            // TODO 这个错误值得重新看一下是因为什么
            return Err(Error::server(format!("[Boya] Response: {}", status.errmsg)));
        }
        if status.status != "0" {
            return Err(Error::server(format!("[Boya] Response: {}", status.errmsg)));
        }

        // 刷新 Token 时效
        self.cred.refresh(Location::Boya);

        Ok(res)
    }
}
