use reqwest::Method;
use serde_json::Value;

use crate::{api::cloud::CreateRes, error::Error};
use crate::utils;

use super::data::{Body, Dir, Item, Root, RootDir, MoveRes};

impl super::CloudApi {
    /// Get root directory by [Root]
    pub async fn get_root_dir(&self, root: Root) -> crate::Result<Vec<RootDir>> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/entry-doc-lib";
        let query = root.as_query();
        let body = Body::Query(&query);
        let res = self.universal_request(Method::GET, url, &body).await?;
        let res = serde_json::from_slice::<Vec<RootDir>>(&res)?;
        Ok(res)
    }

    /// Return User Root directory ID
    pub async fn get_user_dir_id(&self) -> crate::Result<String> {
        let res = self.get_root_dir(Root::User).await?;
        let id = res
            .into_iter()
            .next()
            .map(|item| item.id)
            .ok_or_else(|| crate::Error::server("No user dir found").with_label("Cloud"))?;
        Ok(id)
    }

    /// List the contents of a directory by its ID. Contain [Item::id] and [RootDir::id].
    pub async fn list_dir(&self, id: &str) -> crate::Result<Dir> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/list";
        let data = serde_json::json!({
            "by": "name", // time/size
            "docid": id,
            "sort": "asc" // desc
        });
        let body = Body::Json(&data);
        let res = self.universal_request(Method::POST, url, &body).await?;
        let res = serde_json::from_slice::<Dir>(&res)?;
        Ok(res)
    }

    // 内部方法
    /// Get a download URL for a single file.
    ///
    /// **Note**: If you pass a dir, it will return a bad URL.
    async fn get_single_download_url(&self, item: &Item) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/osdownload";
        let data = serde_json::json!({
            "docid": item.id,
            "authtype": "QUERY_STRING",
        });
        let body = Body::Json(&data);
        let bytes = self.universal_request(Method::POST, url, &body).await?;
        let res = utils::parse_by_tag(&bytes, ",\"", "\"")
            .ok_or_else(|| Error::server("Can not get download url").with_label("Cloud"))?;
        Ok(res.to_string())
    }

    // 内部方法
    /// Get a download URL of a zip package for multiple files or a dir.
    ///
    /// **Note**: If you pass a single file(not a dir), it will return a bad URL.
    async fn get_muti_download_url(&self, items: &[&Item]) -> crate::Result<String> {
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
        let body = Body::Json(&data);
        let bytes = self.universal_request(Method::POST, url, &body).await?;
        let raw_url = utils::parse_by_tag(&bytes, "package_address\":\"", "\"")
            .ok_or_else(|| Error::server("Can not get download url").with_label("Cloud"))?;

        // 在获取下载链接请求发出后获取 token, 通过其自动刷新机制保证 token 正常情况是存在的
        let token = self.cred.load().value::<crate::api::Cloud>()?;

        let url = format!("{raw_url}?token={token}");

        Ok(url)
    }

    /// Get a download URL with the indexes of items.
    ///
    /// **Note**: If indexes is empty, it means all items.
    pub async fn get_download_url(
        &self,
        items: &[Item],
        indexes: &[usize],
    ) -> crate::Result<String> {
        let items: Vec<&Item> = if indexes.is_empty() {
            // 全部文件
            items.iter().collect()
        } else {
            indexes.iter().filter_map(|&idx| items.get(idx)).collect()
        };

        if items.is_empty() {
            return Err(Error::parameter("No valid file selected").with_label("Cloud"));
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
    /// Delete a file or directory by its ID. [Item::id]
    pub async fn delete_item(&self, id: &str) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/delete";
        let data = serde_json::json!({
            "docid": id,
        });
        let body = Body::Json(&data);
        self.universal_request(Method::POST, url, &body).await?;
        Ok(())
    }

    /// Rename a file or directory by its ID. [Item::id]
    pub async fn rename_item(&self, id: &str, new: &str) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/rename";
        let data = serde_json::json!({
            "docid": id,
            "name": new,
            "ondup": 1
        });
        let body = Body::Json(&data);
        self.universal_request(Method::POST, url, &body).await?;
        Ok(())
    }

    /// Move a file or directory by its ID. [Item::id]
    pub async fn move_item(&self, dir: &str, id: &str) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/move";
        let data = serde_json::json!({
            "destparent": dir,
            "docid": id,
            "ondup": 1
        });
        let body = Body::Json(&data);
        let bytes = self.universal_request(Method::POST, url, &body).await?;

        let res = serde_json::from_slice::<MoveRes>(&bytes)?;
        Ok(res.id)
    }

    /// Create directory in given parent directory with name. [Item::id]
    pub async fn create_dir(&self, dir: &str, name: &str) -> crate::Result<CreateRes> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/create";
        let data = serde_json::json!({
            "docid": dir,
            "name": name,
            "ondup": 1
        });
        let body = Body::Json(&data);
        let bytes = self.universal_request(Method::POST, url, &body).await?;

        let res = serde_json::from_slice::<CreateRes>(&bytes)?;
        Ok(res)
    }

    /// Get a suggested name for a new directory in given parent directory. [Item::id]
    ///
    /// Usually for [super::CloudApi::create_dir]
    pub async fn get_suggest_name(&self, dir: &str) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/getsuggestname";
        let data = serde_json::json!({
            "docid": dir,
            "name": "新建文件夹",
        });
        let body = Body::Json(&data);
        let bytes = self.universal_request(Method::POST, url, &body).await?;

        let res = utils::parse_by_tag(&bytes, ":\"", "\"")
            .ok_or_else(|| Error::server("Can not get suggest name").with_label("Cloud"))?
            .to_string();
        Ok(res)
    }
}
