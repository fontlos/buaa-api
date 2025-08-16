use serde::Deserialize;

use crate::api::Location;
use crate::crypto;
use crate::error::Error;

#[derive(Deserialize)]
struct SpocState {
    code: u32,
    msg: Option<String>,
}

impl super::SpocApi {
    pub async fn universal_request(&self, url: &str, query: &str) -> crate::Result<String> {
        if self.cred.load().spoc_token.is_expired() {
            self.login().await?;
        }
        let cred = self.cred.load();
        let token = match cred.spoc_token.value() {
            Some(t) => t,
            None => return Err(Error::auth_expired(Location::Spoc)),
        };
        // 逆向出来的密钥和初始向量, 既然写死了为什么不用 ECB 模式啊
        let ase_key = crate::consts::SPOC_AES_KEY;
        let ase_iv = crate::consts::SPOC_AES_IV;
        let body = serde_json::json!({
            "param": crypto::aes::aes_encrypt_cbc(query.as_bytes(), ase_key, ase_iv)
        });
        let token = format!("Inco-{token}");
        let res = self
            .post(url)
            .header("Token", &token)
            .json(&body)
            .send()
            .await?;
        let res = res.text().await?;
        let status = serde_json::from_str::<SpocState>(&res)?;
        if status.code != 200 {
            return Err(Error::server(format!(
                "[Spoc] Response: {}",
                status.msg.unwrap_or("Unknown Error".into())
            )));
        }
        Ok(res)
    }
}
