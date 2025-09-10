use serde_json::Value;

use crate::api::Location;
use crate::error::Error;
use crate::utils;

use super::data::{CloudDir, CloudItem, CloudRoot, CloudRootDir};

impl super::CloudApi {
    /// Get root directory by [CloudRoot]
    pub async fn get_root_dir(&self, root: CloudRoot) -> crate::Result<Vec<CloudRootDir>> {
        let token = self.token().await?;
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/entry-doc-lib";
        let root = root.as_str();
        let query: &[(&str, &str)] = if root.is_empty() {
            &[("sort", "doc_lib_name"), ("direction", "asc")]
        } else {
            &[
                ("sort", "doc_lib_name"),
                ("direction", "asc"),
                ("type", root),
            ]
        };
        let res = self
            .get(url)
            .bearer_auth(token)
            .query(&query)
            .send()
            .await?
            .bytes()
            .await?;
        println!("res: {}", String::from_utf8_lossy(&res));
        let res = serde_json::from_slice::<Vec<CloudRootDir>>(&res)?;
        Ok(res)
    }

    /// Return User Root directory ID
    pub async fn get_user_dir_id(&self) -> crate::Result<String> {
        let res = self.get_root_dir(CloudRoot::User).await?;
        let id = res
            .into_iter()
            .next()
            .map(|item| item.id)
            .ok_or_else(|| crate::Error::server("[Cloud] No user dir found"))?;
        Ok(id)
    }

    /// List the contents of a directory by its ID. Contain [CloudItem::id] and [CloudRootDir::id].
    pub async fn list_dir(&self, id: &str) -> crate::Result<CloudDir> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/list";
        let data = serde_json::json!({
            "by": "name", // time/size
            "docid": id,
            "sort": "asc" // desc
        });
        let res = self.universal_request(url, &data).await?.bytes().await?;
        let res = serde_json::from_slice::<CloudDir>(&res)?;

        Ok(res)
    }

    /// Get a download URL for a single file.
    ///
    /// **Note**: If you pass a dir, it will return a bad URL.
    async fn get_single_download_url(&self, item: &CloudItem) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/osdownload";
        let data = serde_json::json!({
            "docid": item.id,
            "authtype": "QUERY_STRING",
        });
        let bytes = self.universal_request(url, &data).await?.bytes().await?;
        let res = match utils::parse_by_tag(&bytes, ",\"", "\"") {
            Some(url) => url,
            None => {
                return Err(Error::server("[Cloud] Can not get download url"));
            }
        };

        Ok(res.to_string())
    }

    /// Get a download URL of a zip package for multiple files or a dir.
    ///
    /// **Note**: If you pass a single file(not a dir), it will return a bad URL.
    async fn get_muti_download_url(&self, items: &[&CloudItem]) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/open-doc/v1/file-download";
        let ids: Vec<Value> = items
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
        let bytes = self.universal_request(url, &data).await?.bytes().await?;
        let raw_url = match utils::parse_by_tag(&bytes, "package_address\":\"", "\"") {
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

    /// Get a download URL with the indexes of items.
    ///
    /// **Note**: If indexes is empty, it means all items.
    pub async fn get_download_url(
        &self,
        items: &[CloudItem],
        indexes: &[usize],
    ) -> crate::Result<String> {
        let items: Vec<&CloudItem> = if indexes.is_empty() {
            // 全部文件
            items.iter().collect()
        } else {
            indexes.iter().filter_map(|&idx| items.get(idx)).collect()
        };

        if items.is_empty() {
            return Err(Error::Other("No valid file selected".into()));
        }

        // 下载单个文件只能用这个, 不然得到的链接无法使用
        // 但如果下载单个文件夹不能用这个, 不然得到的链接也无法使用
        if items.len() == 1 && !items[0].is_dir() {
            return self.get_single_download_url(items[0]).await;
        } else {
            return self.get_muti_download_url(&items).await;
        }
    }

    // 重复删掉文件也不会报错
    /// Delete a file or directory by its ID. [CloudItem::id]
    pub async fn delete_item(&self, id: &str) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/delete";
        let data = serde_json::json!({
            "docid": id,
        });
        self.universal_request(url, &data).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[tokio::test]
    async fn test_get_url() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();

        let user_dir = cloud.get_user_dir_id().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();
        let url = cloud.get_download_url(&list.files, &[0]).await.unwrap();

        println!("Download URL: {url}");

        context.save_auth("./data");
    }

    #[tokio::test]
    async fn test_delete() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir_id().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();

        cloud.delete_item(&list.files[0].id).await.unwrap();

        context.save_auth("./data");
    }
}
