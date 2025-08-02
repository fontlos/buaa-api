use serde_json::Value;

use crate::Error;

impl super::CloudApi {
    pub async fn universal_request(&self, url: &str, data: &Value) -> crate::Result<String> {
        // 首先尝试获取 token, 如果没有就可以直接返回了
        let cred = &self.cred.load().cloud_token;
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && cred.is_expired() {
            self.login().await?;
        }
        let token = match cred.value() {
            Some(t) => t,
            None => return Err(Error::ApiError("No Cloud Token".to_string())),
        };

        let res = self.post(url).bearer_auth(token).json(data).send().await?;
        let text = res.text().await?;

        Ok(text)
    }
}
