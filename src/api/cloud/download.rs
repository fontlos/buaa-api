use crate::utils;

use super::utils::CloudItem;

impl super::CloudAPI {
    /// Get a download URL for a single file.
    pub async fn get_download_url(&self, item: &CloudItem) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/osdownload";
        let data = serde_json::json!({
            "docid": item.id,
            "authtype": "QUERY_STRING",
        });
        let text = self.universal_request(&url, &data).await?;
        let res = match utils::get_value_by_lable(&text, ",\"", "\"") {
            Some(url) => url,
            None => return Err(crate::Error::APIError("Failed to get download url".to_string())),
        };

        Ok(res.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[tokio::test]
    async fn test_get_url() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();
        cloud.login().await.unwrap();

        let user_dir = cloud.get_user_dir_id().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();
        let download_url = cloud.get_download_url(&list.files[0]).await.unwrap();

        println!("download_url: {download_url}");

        context.save_auth("./data");
    }
}