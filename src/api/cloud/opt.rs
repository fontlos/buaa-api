use reqwest::Method;
#[cfg(feature = "multipart")]
use reqwest::multipart::{Form, Part};
use serde_json::Value;

use crate::error::Error;
use crate::utils;

use super::data::{
    Body, CreateRes, Dir, Item, MoveRes, RecycleDir, Res, Root, RootDir, Share, SizeRes, UploadArgs,
};

impl super::CloudApi {
    // 这种返回数组类型的如果参数错误都会在上层直接触发 400 错误,
    // 而不是详细的 Json 错误, 所以不能用 Res<T> 包裹
    /// Get root directory by [Root]
    pub async fn get_root_dir(&self, root: Root) -> crate::Result<Vec<RootDir>> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/entry-doc-lib";
        let query = root.as_query();
        let body = Body::Query(&query);
        let bytes = self.universal_request(Method::GET, url, &body).await?;
        let res = serde_json::from_slice::<Vec<RootDir>>(&bytes)?;
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
        let bytes = self.universal_request(Method::POST, url, &body).await?;
        let res = serde_json::from_slice::<Res<Dir>>(&bytes)?;
        res.unpack_with(|r| r, "Can not get dir list")
    }

    // 下载相关的参数错误也会在上层触发 400 错误
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
    /// Delete a file or directory to recycle bin by its ID. [Item::id]
    ///
    /// **Note**: Delete multiple files need call multiple times.
    pub async fn delete_item(&self, id: &str) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/delete";
        let data = serde_json::json!({
            "docid": id,
        });
        let body = Body::Json(&data);
        self.universal_request(Method::POST, url, &body).await?;
        Ok(())
    }

    // 重复删掉文件也不会报错
    /// Delete a file or directory forever by its ID. [Item::id]
    ///
    /// **Note**: Delete multiple files need call multiple times.
    pub async fn delete_recycle_item(&self, id: &str) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/recycle/delete";
        let data = serde_json::json!({
            "docid": id,
        });
        let body = Body::Json(&data);
        self.universal_request(Method::POST, url, &body).await?;
        Ok(())
    }

    // 重命名不存在的文件会在上层触发 400 错误
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
    ///
    /// **Note**: Move multiple files need call multiple times.
    pub async fn move_item(&self, dir: &str, id: &str) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/move";
        let data = serde_json::json!({
            "destparent": dir,
            "docid": id,
            "ondup": 1
        });
        let body = Body::Json(&data);
        let bytes = self.universal_request(Method::POST, url, &body).await?;
        let res = serde_json::from_slice::<Res<MoveRes>>(&bytes)?;
        res.unpack_with(|r| r.id, "Can not move item")
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
        let res = serde_json::from_slice::<Res<CreateRes>>(&bytes)?;
        res.unpack_with(|r| r, "Can not create dir")
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

    /// Get the size of a file or directory by its ID. [Item::id]
    pub async fn get_item_size(&self, id: &str) -> crate::Result<SizeRes> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/size";
        let data = serde_json::json!({
            "docid": id,
            "onlyrecycle": false
        });
        let body = Body::Json(&data);
        let bytes = self.universal_request(Method::POST, url, &body).await?;
        let res = serde_json::from_slice::<Res<SizeRes>>(&bytes)?;
        res.unpack_with(|r| r, "Can not get item size")
    }

    /// Check hash before upload
    pub async fn check_hash(&self, md5: &str, length: u64) -> crate::Result<bool> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/predupload";
        let data = serde_json::json!({
            "slice_md5": md5,
            "length": length
        });
        let body = Body::Json(&data);
        let bytes = self.universal_request(Method::POST, url, &body).await?;

        #[derive(serde::Deserialize)]
        struct _Res {
            #[serde(rename = "match")]
            status: bool,
        }

        let res = serde_json::from_slice::<Res<_Res>>(&bytes)?;
        res.unpack_with(|r| r.status, "Can not check hash")
    }

    /// Fast upload file if hash exists
    pub async fn fast_upload(
        &self,
        dir: &str,
        name: &str,
        length: u64,
        md5: &str,
        crc32: &str,
    ) -> crate::Result<bool> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/dupload";
        let data = serde_json::json!({
            "client_mtime": utils::get_time_millis(),
            "crc32": crc32,
            "csflevel": 0,
            "docid": dir,
            "length": length,
            "md5": md5,
            "name": name,
            "ondup": 1
        });
        let body = Body::Json(&data);
        let bytes = self.universal_request(Method::POST, url, &body).await?;

        #[derive(serde::Deserialize)]
        struct _Res {
            #[serde(rename = "success")]
            status: bool,
        }

        let res = serde_json::from_slice::<Res<_Res>>(&bytes)?;
        res.unpack_with(|r| r.status, "Can not fast upload")
    }

    /// Get upload authorization
    ///
    /// **Note**: You need [super::CloudApi::upload()] for final upload which need enable `multipart` feature.
    pub async fn upload_auth(
        &self,
        dir: &str,
        name: &str,
        length: u64,
    ) -> crate::Result<UploadArgs> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/osbeginupload";
        let data = serde_json::json!({
            "client_mtime": utils::get_time_millis(),
            "docid": dir,
            "length": length,
            "name": name,
            "ondup": 1,
            "reqmethod": "POST",
            "usehttps": true,
        });
        let body = Body::Json(&data);
        let bytes = self.universal_request(Method::POST, url, &body).await?;
        let res = serde_json::from_slice::<Res<UploadArgs>>(&bytes)?;
        res.unpack_with(|r| r, "Can not get upload auth")
    }

    /// Upload file with given [UploadArgs] and file part
    ///
    /// **Note**: MIME of `part` is allready set internally.
    #[cfg(feature = "multipart")]
    pub async fn upload(&self, args: UploadArgs, part: Part) -> crate::Result<()> {
        let auth = args.auth;
        let form = Form::new()
            .text("AWSAccessKeyId", "bhtenant")
            // 似乎只在 part 中设置即可
            // .text("Content-Type", "application/octet-stream")
            .text("Policy", auth.policy)
            .text("Signature", auth.signature)
            .text("key", auth.key)
            .part("file", part.mime_str("application/octet-stream").unwrap());
        let res = self.client.post(auth.url).multipart(form).send().await?;

        let status = res.status().as_u16();
        if status != 204 {
            return Err(Error::server("Upload failed")
                .with_label("Cloud")
                .with_source(format!("HTTP status: {}", status)));
        }

        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/osendupload";
        let data = serde_json::json!({
            "csflevel": 0,
            "docid": args.id,
            "rev": args.hash,
        });
        let body = Body::Json(&data);
        let bytes = self.universal_request(Method::POST, url, &body).await?;

        // editor, modified 字段没有任何用, 暂时不解析
        let res = serde_json::from_slice::<Res<()>>(&bytes)?;

        if res.cause.is_some() {
            let source = format!(
                "Server err: {}, msg: {}",
                res.cause.unwrap_or_default(),
                res.message.unwrap_or_default()
            );
            return Err(Error::server("Can not finalize upload")
                .with_label("Cloud")
                .with_source(source));
        }
        Ok(())
    }

    // 传入不存在的 id 会在上层触发 400 错误
    /// Get share record by [Item::id]
    pub async fn share_record(&self, id: &str) -> crate::Result<Vec<Share>> {
        let url = format!(
            "https://bhpan.buaa.edu.cn/api/shared-link/v1/document/folder/{}?type=anonymous",
            id.replace(':', "%3A").replace('/', "%2F")
        );
        let body = Body::<'_, ()>::None;
        let bytes = self.universal_request(Method::GET, &url, &body).await?;
        let res = serde_json::from_slice::<Vec<Share>>(&bytes)?;
        Ok(res)
    }

    /// Create a share ID for given [Share]. Call [Item::to_share()] to get a [Share] from [Item].
    ///
    /// **Note**: The share link can be formed as `https://bhpan.buaa.edu.cn/link/{ID}`.
    pub async fn share_item(&self, share: &Share) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/shared-link/v1/document/anonymous";

        let body = Body::Json(&share);
        let bytes = self.universal_request(Method::POST, url, &body).await?;

        #[derive(serde::Deserialize)]
        struct _Res {
            id: String,
        }

        let res = serde_json::from_slice::<Res<_Res>>(&bytes)?;
        res.unpack_with(|r| r.id, "Can not create share link")
    }

    /// Update share link by Share ID and new [Share]
    pub async fn share_update(&self, id: &str, share: &Share) -> crate::Result<()> {
        let url = format!("https://bhpan.buaa.edu.cn/api/shared-link/v1/document/anonymous/{id}");
        let body = Body::Json(&share);
        self.universal_request(Method::PUT, &url, &body).await?;
        Ok(())
    }

    /// Delete share link by Share ID
    pub async fn share_delete(&self, id: &str) -> crate::Result<()> {
        let url = format!("https://bhpan.buaa.edu.cn/api/shared-link/v1/document/anonymous/{id}");
        let body = Body::<'_, ()>::None;
        self.universal_request(Method::DELETE, &url, &body).await?;
        Ok(())
    }

    /// List recycle bin contents of user's personal directory
    pub async fn list_recycle(&self) -> crate::Result<RecycleDir> {
        let id = self.get_user_dir_id().await?;
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/recycle/list";
        let data = serde_json::json!({
            "by": "time", // name/size
            "docid": id,
            "sort": "desc" // asc
        });
        let body = Body::Json(&data);
        let res = self.universal_request(Method::POST, url, &body).await?;
        let res = serde_json::from_slice::<Res<RecycleDir>>(&res)?;
        res.unpack_with(|r| r, "Can not get recycle dir")
    }
}
