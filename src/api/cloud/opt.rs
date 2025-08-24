use crate::api::Location;
use crate::error::Error;
use crate::utils;

use super::data::{CloudItem, CloudDir, CloudRootDir};

impl super::CloudApi {
    /// Get directory by category, possible categories:
    /// - `""`: All directories
    /// - `"user_doc_lib"`: User's personal directory
    /// - `"shared_user_doc_lib"`: Shared directory
    /// - `"department_doc_lib"`: Department directory
    /// - `"custom_doc_lib"`: Other directory
    pub async fn get_root_dir(&self, category: &str) -> crate::Result<Vec<CloudRootDir>> {
        let token = self.token().await?;
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/entry-doc-lib";
        let mut query = vec![("sort", "doc_lib_name"), ("direction", "asc")];
        if !category.is_empty() {
            query.push(("type", category));
        }
        let res = self
            .get(url)
            .bearer_auth(token)
            .query(&query)
            .send()
            .await?
            .json::<Vec<CloudRootDir>>()
            .await?;
        Ok(res)
    }

    /// Return All Type Root directory
    pub async fn get_all_dir(&self) -> crate::Result<Vec<CloudRootDir>> {
        self.get_root_dir("").await
    }

    /// Return User Root directory ID
    pub async fn get_user_dir_id(&self) -> crate::Result<String> {
        let res = self.get_root_dir("user_doc_lib").await?;
        let id = res
            .into_iter()
            .next()
            .map(|item| item.id)
            .ok_or_else(|| crate::Error::server("[Cloud] No user dir found"))?;
        Ok(id)
    }

    pub async fn list_dir(&self, id: &str) -> crate::Result<CloudDir> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/list";
        let data = serde_json::json!({
            "by": "name", // time/size
            "docid": id,
            "sort": "asc" // desc
        });
        let text = self.universal_request(url, &data).await?;
        let res = serde_json::from_str::<CloudDir>(&text)?;

        Ok(res)
    }

    /// Get a download URL for a single file.
    pub async fn get_download_url(&self, item: &CloudItem) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/osdownload";
        let data = serde_json::json!({
            "docid": item.id,
            "authtype": "QUERY_STRING",
        });
        let text = self.universal_request(url, &data).await?;
        let res = match utils::get_value_by_lable(&text, ",\"", "\"") {
            Some(url) => url,
            None => {
                return Err(Error::server("[Cloud] Can not get download url"));
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
                // 同样是下载, 单个文件就用完整 id, 多个文件就用文件 id, 那证明文件 id 就够用了, 这什么 ** 设计
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
        let text = self.universal_request(url, &data).await?;
        let raw_url = match utils::get_value_by_lable(&text, "address\":\"", "\"") {
            Some(url) => url,
            None => {
                return Err(Error::server("[Cloud] Can not get download url"));
            }
        };

        // 在获取下载链接请求发出后获取 cred, 通过其自动刷新机制保证 token 正常情况是存在的
        let cred = match self.cred.load().cloud_token.value() {
            Some(token) => token,
            None => return Err(Error::auth_expired(Location::Cloud)),
        };

        let url = format!("{raw_url}?token={cred}");

        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[tokio::test]
    async fn test_get_list() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();

        let dir = cloud.get_user_dir_id().await.unwrap();
        let list = cloud.list_dir(&dir).await.unwrap();

        println!("list: {list:?}");
    }

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
