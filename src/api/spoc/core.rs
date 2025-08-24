use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::api::Location;
use crate::crypto;
use crate::error::Error;

use super::_SpocRes;

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

        self.cred.set(Location::Spoc, token.to_string());
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
        if self.cred.load().spoc_token.is_expired() {
            self.login().await?;
        }
        let cred = self.cred.load();
        let token: &String = match cred.spoc_token.value() {
            Some(t) => t,
            None => return Err(Error::auth_expired(Location::Spoc)),
        };
        // 逆向出来的密钥和初始向量, 既然写死了为什么不用 ECB 模式啊
        let ase_key = crate::consts::SPOC_AES_KEY;
        let ase_iv = crate::consts::SPOC_AES_IV;
        let query = serde_json::to_vec(query)?;
        let body = serde_json::json!({
            "param": crypto::aes::aes_encrypt_cbc(&query, ase_key, ase_iv)
        });
        let token = format!("Inco-{token}");
        let res = self
            .post(url)
            .header("Token", &token)
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
