use bytes::Bytes;
use reqwest::Method;
use serde::Serialize;

use crate::api::{Spoc, Sso};
use crate::crypto;
use crate::error::Error;

use super::data::Payload;

// 逆向出来的密钥和初始向量, 用于 AES 加密请求体,
// 不过既然写死了为什么不用 ECB 而用 CBC 模式啊
/// From hard-coded in JS
/// 2025.04.22
const SPOC_AES_KEY: &[u8] = b"inco12345678ocni";
const SPOC_AES_IV: &[u8] = b"ocni12345678inco";

impl super::SpocApi {
    /// # Login to SpocApi
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        let res = self
            .client
            .get("https://spoc.buaa.edu.cn/spocnewht/cas")
            .send()
            .await?;
        if res.url().as_str().contains("https://sso.buaa.edu.cn/login") {
            return Err(Error::server("Redirect failed").with_label("Spoc"));
        }
        let mut query = res.url().query_pairs();
        let token = query
            .next()
            .and_then(|t| if t.0 == "token" { Some(t.1) } else { None })
            .ok_or_else(|| Error::server("Login failed. No token").with_label("Spoc"))?;
        // 再次调用 next 获取 refreshToken, 但我们用不着, 使用我们自己的机制刷新登陆状态

        // 提前加上前缀
        self.cred.update(|s| {
            s.update::<Spoc>(format!("Inco-{token}"));
        });
        Ok(())
    }

    /// # Universal Request for SpocApi
    ///
    /// **Note**: You should use other existing APIs first.
    ///
    /// If the API you need but is not implemented, you can extend it with this universal request API.
    pub async fn universal_request<P>(
        &self,
        url: &str,
        method: Method,
        payload: Payload<'_, P>,
    ) -> crate::Result<Bytes>
    where
        P: Serialize + ?Sized,
    {
        let cred = self.cred.load();
        if cred.is_expired::<Spoc>() {
            self.login().await?;
        }
        let token = cred.value::<Spoc>()?;

        let req = self.client.request(method, url).header("Token", token);
        let req = match payload {
            Payload::Query(q) => req.query(q),
            // 它们是不是把这个玩意忘了, 做了这么多加密结果只有一个接口在用
            Payload::Json(j) => {
                // 构造请求体, 使用 AES 加密请求参数, Base64 编码
                let aes = crypto::aes::Aes128::new(SPOC_AES_KEY);
                // TODO: 考虑直接序列化到加密器中
                let data = serde_json::to_vec(j)?;
                let data = aes.encrypt_cbc(&data, SPOC_AES_IV);
                let data = crypto::encode_base64(data);
                let json = serde_json::json!({ "param": data });
                req.json(&json)
            }
        };

        let res = req.send().await?.bytes().await?;
        Ok(res)
    }
}
