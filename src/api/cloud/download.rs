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
            None => {
                return Err(crate::Error::APIError("No url".to_string()));
            }
        };

        Ok(res.to_string())
    }

    /// Get a download URL of a zip package for multiple files.
    pub async fn get_muti_download_url(&self, items: &[CloudItem]) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/open-doc/v1/file-download";
        let ids: Vec<_> = items
            .iter()
            .map(|item| {
                // 从文件路径 id 反向找到文件 id, 找不到就用原 id, 这不会引起错误, 只会导致无法下载这个文件
                // 为什么同样是下载, 单个文件就用完整 id, 多个文件就用文件 id, 那证明文件 id 就够用了, 为什么这样设计
                let file_id = match item.id.rfind('/') {
                    Some(idx) => &item.id[idx + 1..],
                    None => &item.id,
                };
                serde_json::json!({ "id": file_id })
            })
            .collect();
        let data = serde_json::json!({
            "name": "download.zip",
            "doc": ids
        });
        let text = self.universal_request(&url, &data).await?;
        let raw_url = match utils::get_value_by_lable(&text, "address\":\"", "\"") {
            Some(url) => url,
            None => {
                return Err(crate::Error::APIError("No url".to_string()));
            }
        };

        // 在获取下载链接请求发出后获取 cred, 通过其自动刷新机制保证 token 正常情况是存在的
        let cred = match self.cred.load().cloud_token.value() {
            Some(token) => token,
            None => return Err(crate::Error::APIError("No cloud token".to_string())),
        };

        let url = format!("{}?token={}", raw_url, cred);

        Ok(url)
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
        // let download_url = cloud.get_download_url(&list.files[0]).await.unwrap();
        let download_url = cloud.get_muti_download_url(&list.files).await.unwrap();

        println!("download_url: {download_url}");

        context.save_auth("./data");
    }
}
