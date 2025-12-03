use log::trace;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::api::{Spoc, Sso};
use crate::crypto;
use crate::error::Error;

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
    ///
    /// **Note**: Type of `T` is the `content` field in JSON response.
    pub async fn universal_request<Q, T>(&self, url: &str, query: &Q) -> crate::Result<T>
    where
        Q: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let cred = self.cred.load();
        if cred.is_expired::<Spoc>() {
            self.login().await?;
        }
        let token = cred.value::<Spoc>()?;

        // 初始化 AES
        let aes = crypto::aes::Aes128::new(SPOC_AES_KEY);

        // 构造请求体, 使用 AES 加密请求参数, Base64 编码
        let body = serde_json::to_vec(query)?;
        let body = aes.encrypt_cbc(&body, SPOC_AES_IV);
        let body = crypto::encode_base64(body);
        let body = serde_json::json!({
            "param": body
        });

        let res = self
            .client
            .post(url)
            .header("Token", token)
            .json(&body)
            .send()
            .await?
            .bytes()
            .await?;

        let res = serde_json::from_slice::<Res<T>>(&res)?;

        // 凭据过期 code 也是 200, 那你这 code 有什么用啊
        if res.code != 200 {
            trace!("URL: {}, Query: {}", url, serde_json::to_string(&query)?);
            let source = format!("Status Code: {}. Error Message: {:?}", res.code, res.msg);
            return Err(Error::server("Operation failed")
                .with_label("Spoc")
                .with_source(source));
        }

        Ok(res.content)
    }
}

#[derive(Deserialize)]
struct Res<T> {
    code: u32,
    msg: Option<String>,
    content: T,
}
