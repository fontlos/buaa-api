use reqwest::Method;
use serde_json::Value;

use crate::error::Error;
use crate::utils;

use super::data::{
    Dir, Item, Payload, Res, Root, RootDir, Share, SizeRes, UploadArgs, UploadAuth,
    parse_error,
};

impl super::CloudApi {
    /// # Get root directory
    pub async fn get_root_dir(&self, root: Root) -> crate::Result<Vec<RootDir>> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/entry-doc-lib";
        let query = root.as_query();
        let payload = Payload::Query(&query);
        let bytes = self.universal_request(Method::GET, url, &payload).await?;
        // 纯数组无法放进 Res 结构体
        let res = serde_json::from_slice::<Vec<RootDir>>(&bytes)
            .map_err(|e| parse_error("Can not get root dir", &bytes, &e))?;
        Ok(res)
    }

    /// # Get User Root directory [RootDir::id]
    pub async fn get_user_dir_id(&self) -> crate::Result<String> {
        let res = self.get_root_dir(Root::User).await?;
        let id = res
            .into_iter()
            .next()
            .map(|item| item.id)
            .ok_or_else(|| Error::server("No user dir found").with_label("Cloud"))?;
        Ok(id)
    }

    /// # List the contents of a directory
    ///
    /// - Input: Directory [Item::id] or [RootDir::id]
    pub async fn list_dir(&self, id: &str) -> crate::Result<Dir> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/list";
        let json = serde_json::json!({
            "by": "name", // time/size
            "docid": id,
            "sort": "asc" // desc
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res: Dir = Res::parse(&bytes, "Can not get dir list")?;
        Ok(res)
    }

    /// # Get the size of an item
    ///
    /// - Input: [Item::id]
    pub async fn get_item_size(&self, id: &str) -> crate::Result<SizeRes> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/size";
        let json = serde_json::json!({
            "docid": id,
            "onlyrecycle": false
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res: SizeRes = Res::parse(&bytes, "Can not get item size")?;
        Ok(res)
    }

    /// # Get a suggested name when name conflict
    ///
    /// - Input:
    ///     - dir: Parent directory [Item::id]
    ///     - name: Desired name
    ///
    /// **Note**: For [super::CloudApi::create_dir] etc.
    pub async fn get_suggest_name(&self, dir: &str, name: &str) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/getsuggestname";
        let json = serde_json::json!({
            "docid": dir,
            "name": name,
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res = utils::parse_by_tag(&bytes, "\"name\":\"", "\"")
            .ok_or_else(|| parse_error("Can not get suggest name", &bytes, &"No 'name' field"))?;
        Ok(res.to_string())
    }

    /// # Create directory
    ///
    /// - Input:
    ///     - dir: Parent directory [Item::id]
    ///     - name: Desired directory name
    pub async fn create_dir(&self, dir: &str, name: &str) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/create";
        let json = serde_json::json!({
            "docid": dir,
            "name": name,
            "ondup": 1
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res = utils::parse_by_tag(&bytes, "\"docid\":\"", "\"")
            .ok_or_else(|| parse_error("Can not create dir", &bytes, &"No 'docid' field"))?;
        Ok(res.to_string())
    }

    // 重命名不存在的文件会在上层触发 400 错误
    /// # Rename an item
    ///
    /// - Input:
    ///     - id: Item [Item::id]
    ///     - new: New name
    pub async fn rename_item(&self, id: &str, new: &str) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/rename";
        let json = serde_json::json!({
            "docid": id,
            "name": new,
            "ondup": 1
        });
        let payload = Payload::Json(&json);
        self.universal_request(Method::POST, url, &payload).await?;
        Ok(())
    }

    /// # Move an item [Item::id]
    ///
    /// - Input:
    ///    - dir: Destination directory [Item::id]
    ///    - id: Item [Item::id]
    ///
    /// **Note**: Move multiple files need call multiple times.
    pub async fn move_item(&self, dir: &str, id: &str) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/move";
        let json = serde_json::json!({
            "destparent": dir,
            "docid": id,
            "ondup": 1
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res = utils::parse_by_tag(&bytes, "\"docid\":\"", "\"")
            .ok_or_else(|| parse_error("Can not move item", &bytes, &"No 'docid' field"))?;
        Ok(res.to_string())
    }

    /// # Copy an item [Item::id]
    ///
    /// - Input:
    ///    - dir: Destination directory [Item::id]
    ///    - id: Item [Item::id]
    ///
    /// **Note**: Copy multiple files need call multiple times.
    pub async fn copy_item(&self, dir: &str, id: &str) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/copy";
        let json = serde_json::json!({
            "destparent": dir,
            "docid": id,
            "ondup": 1
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res = utils::parse_by_tag(&bytes, "\"docid\":\"", "\"")
            .ok_or_else(|| parse_error("Can not copy item", &bytes, &"No 'docid' field"))?;
        Ok(res.to_string())
    }

    // 重复删掉文件也不会报错
    /// # Delete an item to recycle bin
    ///
    /// - Input: [Item::id]
    ///
    /// **Note**: Delete multiple files need call multiple times.
    pub async fn delete_item(&self, id: &str) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/delete";
        let json = serde_json::json!({
            "docid": id,
        });
        let payload = Payload::Json(&json);
        self.universal_request(Method::POST, url, &payload).await?;
        Ok(())
    }

    /// # List recycle bin contents of user's personal directory
    pub async fn list_recycle(&self) -> crate::Result<Dir> {
        let id = self.get_user_dir_id().await?;
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/recycle/list";
        let json = serde_json::json!({
            "by": "time", // name/size
            "docid": id,
            "limit": 100,
            "sort": "desc", // asc
            "start": 0
        });
        let payload = Payload::Json(&json);
        let res = self.universal_request(Method::POST, url, &payload).await?;
        let res: Dir = Res::parse(&res, "Can not get recycle dir")?;
        Ok(res)
    }

    /// # Delete an item forever in recycle bin
    ///
    /// - Input: [Item::id]
    ///
    /// **Note**: Delete multiple files need call multiple times.
    pub async fn delete_recycle_item(&self, id: &str) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/recycle/delete";
        let json = serde_json::json!({
            "docid": id,
        });
        let payload = Payload::Json(&json);
        self.universal_request(Method::POST, url, &payload).await?;
        Ok(())
    }

    /// # Restore an item from recycle bin
    ///
    /// - Input: [Item::id]
    ///
    /// **Note**: Restore multiple files need call multiple times.
    pub async fn restore_recycle_item(&self, id: &str) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/recycle/restore";
        let json = serde_json::json!({
            "docid": id,
            "ondup": 1
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;

        let res = utils::parse_by_tag(&bytes, "\"docid\":\"", "\"").ok_or_else(|| {
            parse_error("Can not restore recycle item", &bytes, &"No 'docid' field")
        })?;
        Ok(res.to_string())
    }

    // 传入不存在的 id 会在上层触发 400 错误
    /// # Get item share record
    ///
    /// - Input: [Item::id]
    pub async fn share_record(&self, id: &str) -> crate::Result<Vec<Share>> {
        let url = format!(
            "https://bhpan.buaa.edu.cn/api/shared-link/v1/document/folder/{}?type=anonymous",
            id.replace(':', "%3A").replace('/', "%2F")
        );
        let payload = Payload::<'_, ()>::Empty;
        let bytes = self.universal_request(Method::GET, &url, &payload).await?;
        let res = serde_json::from_slice::<Vec<Share>>(&bytes)?;
        Ok(res)
    }

    /// # Share an item
    ///
    /// - Input: [Share] from [Item::to_share()]
    /// - Output: Share ID
    ///
    /// **Note**: The share link can be formed as `https://bhpan.buaa.edu.cn/link/{ID}`.
    pub async fn share_item(&self, share: &Share) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/shared-link/v1/document/anonymous";
        let payload = Payload::Json(&share);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res = utils::parse_by_tag(&bytes, "\"id\":\"", "\"")
            .ok_or_else(|| parse_error("Can not create share link", &bytes, &"No 'id' field"))?;
        Ok(res.to_string())
    }

    /// # Update share
    ///
    /// - Input:
    ///     - id: Share ID
    ///     - share: Updated [Share]
    pub async fn share_update(&self, id: &str, share: &Share) -> crate::Result<()> {
        let url = format!("https://bhpan.buaa.edu.cn/api/shared-link/v1/document/anonymous/{id}");
        let payload = Payload::Json(&share);
        self.universal_request(Method::PUT, &url, &payload).await?;
        Ok(())
    }

    /// # Delete share
    ///
    /// - Input: Share ID
    pub async fn share_delete(&self, id: &str) -> crate::Result<()> {
        let url = format!("https://bhpan.buaa.edu.cn/api/shared-link/v1/document/anonymous/{id}");
        let payload = Payload::<'_, ()>::Empty;
        self.universal_request(Method::DELETE, &url, &payload)
            .await?;
        Ok(())
    }

    // 下载相关的参数错误也会在上层触发 400 错误
    // 内部方法
    /// Get a download URL for a single file.
    ///
    /// **Note**: If you pass a dir, it will return a bad URL.
    async fn get_single_download_url(&self, item: &Item) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/osdownload";
        let json = serde_json::json!({
            "docid": item.id,
            "authtype": "QUERY_STRING",
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        // 这是 authrequest 数组中的第二个元素
        let res = utils::parse_by_tag(&bytes, ",\"", "\"").ok_or_else(|| {
            parse_error("Can not get download url", &bytes, &"No valid URL found")
        })?;
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
                // 同样是下载, 单个文件就用完整 id, 多个文件就用文件 id, 那证明文件 id 就够用了, 这什么**设计
                let file_id = match item.id.rfind('/') {
                    Some(idx) => &item.id[idx + 1..],
                    None => &item.id,
                };
                serde_json::json!({ "id": file_id })
            })
            .collect();
        let json = serde_json::json!({
            "name": "download.zip",
            "doc": ids
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let raw_url =
            utils::parse_by_tag(&bytes, "\"package_address\":\"", "\"").ok_or_else(|| {
                parse_error(
                    "Can not get download url",
                    &bytes,
                    &"No 'package_address' field",
                )
            })?;

        // 在获取下载链接请求发出后获取 token, 通过其自动刷新机制保证 token 正常情况是存在的
        let token = self.cred.load().value::<crate::api::Cloud>()?;
        let url = format!("{raw_url}?token={token}");
        Ok(url)
    }

    /// # Get a download URL with the indexes of items.
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

    /// # Check whether can upload fast
    ///
    /// - Input: Need call [UploadArgs::compute_mini]
    #[cfg(feature = "multipart")]
    pub async fn upload_fast_check(&self, args: &UploadArgs) -> crate::Result<bool> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/predupload";
        let json = serde_json::json!({
            "slice_md5": args.slice_md5,
            "length": args.length
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let matched = utils::parse_by_tag(&bytes, "\"match\":", "}")
            .ok_or_else(|| parse_error("Can not check hash", &bytes, &"No 'match' field"))?;
        Ok(matched == "true")
    }

    /// # Upload file fast
    ///
    /// - Input: Need call [UploadArgs::compute_full]
    #[cfg(feature = "multipart")]
    pub async fn upload_fast(&self, args: &UploadArgs) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/dupload";
        let json = serde_json::json!({
            "client_mtime": utils::get_time_millis(),
            "crc32": args.crc32,
            "docid": args.dir,
            "length": args.length,
            "md5": args.md5,
            "name": args.name,
            // 文件名冲突则抛出异常
            "ondup": 1
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let success = utils::parse_by_tag(&bytes, "\"success\":", "}")
            .ok_or_else(|| parse_error("Can not upload fast", &bytes, &"No 'success' field"))?;
        if success != "true" {
            return Err(Error::server("Can not upload fast").with_label("Cloud"));
        }
        Ok(())
    }

    /// # Upload small file
    ///
    /// - Input: Need call [UploadArgs::compute_mini]
    ///
    /// **Note**: File size should be less than 5 GiB. Recommended for files smaller than 100 MiB.
    #[cfg(feature = "multipart")]
    pub async fn upload_small(&self, args: &UploadArgs, body: Vec<u8>) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/osbeginupload";
        let json = serde_json::json!({
            "client_mtime": utils::get_time_millis(),
            "docid": args.dir,
            "length": args.length,
            "name": args.name,
            "ondup": 1,
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let auth: UploadAuth = Res::parse(&bytes, "Can not get upload auth")?;
        let req = self.client.put(&auth.authrequest[1]);
        // 0 号是方法, 1 号是 URL, 剩余的是 Header
        let req = auth.authrequest.iter().skip(2).fold(req, |req, header| {
            if let Some((key, value)) = header.split_once(": ") {
                req.header(key, value)
            } else {
                req
            }
        });
        let res = req.body(body).send().await?;
        let status = res.status();
        if !status.is_success() {
            return Err(Error::server("Upload failed")
                .with_label("Cloud")
                .with_source(format!("HTTP status: {}", status.as_u16())));
        }
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/osendupload";
        let json = serde_json::json!({
            "docid": auth.docid,
            "rev": auth.rev,
        });
        let payload = Payload::Json(&json);
        // editor 等无用字段
        let _bytes = self.universal_request(Method::POST, url, &payload).await?;
        Ok(())
    }
}
