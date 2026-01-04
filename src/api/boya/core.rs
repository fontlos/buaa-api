use bytes::Bytes;
use serde::Serialize;

use crate::api::{Boya, Sso};
use crate::error::Error;
use crate::{crypto, utils};

/// From hard-coded in JS
/// 2025.04.22
const BOYA_RSA_KEY: &str = "-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDlHMQ3B5GsWnCe7Nlo1YiG/YmH
dlOiKOST5aRm4iaqYSvhvWmwcigoyWTM+8bv2+sf6nQBRDWTY4KmNV7DBk1eDnTI
Qo6ENA31k5/tYCLEXgjPbEjCK9spiyB62fCT6cqOhbamJB0lcDJRO6Vo1m3dy+fD
0jbxfDVBBNtyltIsDQIDAQAB
-----END PUBLIC KEY-----";

impl super::BoyaApi {
    /// # Login to BoyaApi
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/cas/login
        let login_url = "https://sso.buaa.edu.cn/login?noAutoRedirect=true&service=https%3A%2F%2Fbykc.buaa.edu.cn%2Fsscv%2Fcas%2Flogin";
        // 获取 JSESSIONID
        let res = self.client.get(login_url).send().await?;

        // 从重定向 URL 中获取 token
        let url = res.url().as_str();
        // 自动刷新机制保证了正常情况下不会发生这种情况
        if url == login_url {
            return Err(Error::server("Redirect failed").with_label("Boya"));
        }
        let token = utils::parse_by_tag(url.as_bytes(), "token=", "")
            .ok_or_else(|| Error::server("Login failed. No token").with_label("Boya"))?;
        self.cred.update(|s| {
            s.update::<Boya>(token.to_string());
        });

        Ok(())
    }

    /// # Universal Request for BoyaApi
    ///
    /// **Note**: You should use other existing APIs first.
    ///
    /// If the API you need but is not implemented, you can extend it with this universal request API.
    ///
    /// ## Usage
    ///
    /// - Input:
    ///     - URL: API URL
    ///     - Payload: Serialize JSON
    ///
    /// ## Example
    ///
    /// Locate the following sections in the `app.js`(Windows UA)/`main.js`(Android UA)
    /// by search `setPublicKey` and set breakpoint to debug.
    ///
    /// ```js
    /// ...
    /// y.setPublicKey(b);
    /// var x = c || {}
    ///   , w = JSON.stringify(x)
    /// ...
    /// ```
    ///
    /// You can find `payload` in `w = JSON.stringify(x)`.
    /// And then for `getUserProfile` API
    ///
    /// ```
    /// use serde_json::Value;
    /// let url = "https://bykc.buaa.edu.cn/sscv/getUserProfile";
    /// let payload = serde_json::json!({});
    /// let bytes = self.universal_request(&url, &payload).await?;
    /// let res: Value = serde_json::from_slice(&bytes)?;
    /// ```
    pub async fn universal_request<P>(&self, url: &str, payload: &P) -> crate::Result<Bytes>
    where
        P: Serialize + ?Sized,
    {
        let cred = self.cred.load();
        if cred.is_expired::<Boya>() {
            self.login().await?;
        }
        let token = cred.value::<Boya>()?;

        // 初始化 RNG. 因为 HTTPS 已经保证安全, 这里无需密码学级别的 RNG, 选一个快速的
        let mut rng = crypto::rand::WyRng::new();

        // 初始化 RSA, 设置公钥
        let rsa_cipher = crypto::rsa::RsaPkcs1v15::from_pem(BOYA_RSA_KEY);

        // 初始化 AES, 使用十六位随机密钥
        let aes_key = utils::gen_rand_str(16);
        let aes_key = aes_key.as_bytes();
        let aes_cipher = crypto::aes::Aes128::new(aes_key);

        // 请求头 Ak 参数, 由 AES Key 生成
        let ak = rsa_cipher.encrypt(&mut rng, aes_key);
        let ak = crypto::encode_base64(ak);

        // 查询参数序列化到字节数组
        let date = serde_json::to_vec(payload)?;

        // 请求头 Sk 参数, 由查询参数生成
        let sk = crypto::sha1::Sha1::digest(&date);
        let sk = crypto::bytes2hex(&sk);
        let sk = rsa_cipher.encrypt(&mut rng, sk.as_bytes());
        let sk = crypto::encode_base64(sk);

        // 请求体负载, 由查询参数生成, 使用 AES 加密, Base64 编码
        let body = aes_cipher.encrypt_ecb(&date);
        let body = crypto::encode_base64(body);

        let time = utils::get_time_millis();

        // 现在似乎传一遍 token 就可以了, 留哪个都行
        let res = self
            .client
            .post(url)
            // .header("Auth_token", token)
            .header("Authtoken", token)
            .header("Ak", &ak)
            .header("Sk", &sk)
            .header("Ts", time.to_string())
            // 虽然请求头是 JSON 但是传的是 Base64 的字符串
            .json(&body)
            .send()
            .await?;

        // 去掉响应体两端的引号, 先 Base64 解码, 再 AES 解密, 然后返回原始字节延迟解析
        let bytes = res
            .bytes()
            .await
            .map(|b| crypto::decode_base64(&b[1..b.len() - 1]))
            .map(|b| aes_cipher.decrypt_ecb(&b))
            .map(Bytes::from)?;

        // 刷新 Token 时效
        cred.refresh::<Boya>();

        Ok(bytes)
    }
}
