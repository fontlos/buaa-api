use reqwest::Method;

use crate::Error;
use crate::utils;

use super::data::Payload;

impl super::LiveApi {
    /// # Get User ID
    pub async fn get_user_id(&self) -> crate::Result<String> {
        let url = "https://classroom.msa.buaa.edu.cn/consoleapi/v2/user/group-user";
        let payload = Payload::<()>::Empty;
        //    "code": 10000,
        //    "message": "",
        //    "data":
        let bytes = self.universal_request(url, Method::GET, payload).await?;
        let id = utils::parse_by_tag(&bytes, "\"id\":\"", "\"")
            .ok_or_else(|| Error::parse("Failed to parse user ID".to_string()))?;
        Ok(id.to_string())
    }
}
