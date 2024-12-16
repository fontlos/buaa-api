use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;

use crate::{crypto, Error};

use super::SpocAPI;

#[derive(Deserialize)]
struct SpocState {
    code: u32,
    msg: Option<String>,
}

impl SpocAPI {
    pub async fn universal_request(&self, query: &str, url: &str) -> crate::Result<String> {
        let config = self.config.read().unwrap();
        let token = match &config.spoc_token {
            Some(t) => t,
            None => return Err(Error::APIError("No Token".to_string())),
        };
        // 逆向出来的密钥和初始向量, 既然写死了为什么不用 ECB 模式啊
        let ase_key = "inco12345678ocni";
        let ase_iv = "ocni12345678inco";
        let body = serde_json::json!({
            "param": crypto::aes::aes_encrypt_cbc(query, ase_key, ase_iv)
        });
        let token = format!("Inco-{}", token);
        let mut header = HeaderMap::new();
        header.insert(
            HeaderName::from_bytes(b"Token").unwrap(),
            HeaderValue::from_str(&token).unwrap(),
        );
        let res = self.post(url).headers(header).json(&body).send().await?;
        let res = res.text().await?;
        let status = serde_json::from_str::<SpocState>(&res)?;
        if status.code != 200 {
            return Err(Error::APIError(
                status.msg.unwrap_or("Unknown Error".to_string()),
            ));
        }
        Ok(res)
    }
}
