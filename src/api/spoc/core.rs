use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::api::Location;
use crate::crypto;
use crate::error::Error;

use super::_SpocRes;

// 逆向出来的密钥和初始向量, 用于 AES 加密请求体,
// 不过既然写死了为什么不用 ECB 而用 CBC 模式啊
/// From hard-coded in JS
/// 2025.04.22
const SPOC_AES_KEY: &[u8] = b"inco12345678ocni";
const SPOC_AES_IV: &[u8] = b"ocni12345678inco";

impl super::SpocApi {
    /// # Spoc Login
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().sso.is_expired() {
            self.api::<crate::api::Sso>().login().await?;
        }

        let res = self
            .get("https://spoc.buaa.edu.cn/spocnewht/cas")
            .send()
            .await?;
        if res.url().as_str().contains("https://sso.buaa.edu.cn/login") {
            return Err(Error::auth_expired(Location::Sso));
        }
        let mut query = res.url().query_pairs();
        let token = query
            .next()
            .and_then(|t| if t.0 == "token" { Some(t.1) } else { None })
            .ok_or_else(|| Error::server("[Spoc] Login failed. No token"))?;
        // 再次调用 next 获取 refreshToken, 但我们用不着, 使用我们自己的机制刷新登陆状态

        // 提前加上前缀
        self.cred.set(Location::Spoc, format!("Inco-{token}"));
        Ok(())
    }

    /// # Spoc Universal Request API
    ///
    /// **Note**: You should use other existing APIs first.
    ///
    /// If the API you need but is not implemented, you can extend it with this universal request API.
    ///
    /// **Note**: Type of `T` is the `content` field in JSON response.
    pub async fn universal_request<Q, T>(&self, url: &str, query: &Q) -> crate::Result<T>
    where
        Q: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let cred = &self.cred.load().spoc_token;
        if cred.is_expired() {
            self.login().await?;
        }

        let token = match cred.value() {
            Some(t) => t,
            None => return Err(Error::auth_expired(Location::Spoc)),
        };

        // 初始化 AES
        let aes = crypto::aes::Aes128::new(SPOC_AES_KEY).unwrap();

        // 构造请求体, 使用 AES 加密请求参数, Base64 编码
        let body = serde_json::to_vec(query)?;
        let body = aes.encrypt_cbc(&body, SPOC_AES_IV);
        let body = crypto::encode_base64(body);
        let body = serde_json::json!({
            "param": body
        });

        let res = self
            .post(url)
            .header("Token", token)
            .json(&body)
            .send()
            .await?
            .json::<_SpocRes<T>>()
            .await?;

        if res.code != 200 {
            return Err(Error::server(format!(
                "[Spoc] Response: {}",
                res.msg.unwrap_or("Unknown Error".into())
            )));
        }

        self.cred.refresh(Location::Spoc);

        Ok(res.content)
    }
}
