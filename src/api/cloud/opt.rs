use reqwest::Method;
use serde_json::Value;

use crate::error::Error;
use crate::utils;

use super::data::{
    Dir, Item, Payload, Res, Root, RootDir, Share, Size, UploadArgs, UploadAuth, parse_error,
};

impl super::CloudApi {
    /// # Get root directory. Better call [RootDir::into_item] to use
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

    /// # Get User Root directory
    pub async fn get_user_dir(&self) -> crate::Result<Item> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/owned-doc-lib";
        let payload = Payload::<'_, ()>::Empty;
        let bytes = self.universal_request(Method::GET, url, &payload).await?;
        let [res]: [RootDir; 1] = serde_json::from_slice(&bytes)
            .map_err(|e| parse_error("No user dir found", &bytes, &e))?;
        Ok(res.into_item())
    }

    // 总体来看, `dir` 接口更强大, 几乎支持所有 `file` 操作, 所以尽量统一使用这个

    /// # List the contents of a directory
    pub async fn list_dir(&self, item: &Item) -> crate::Result<Dir> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/list";
        let json = serde_json::json!({
            "by": "name", // time/size
            "docid": item.id,
            "sort": "asc" // desc
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res: Dir = Res::parse(&bytes, "Can not get dir list")?;
        Ok(res)
    }

    /// # Get the size of an item
    pub async fn get_item_size(&self, item: &Item) -> crate::Result<Size> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/size";
        let json = serde_json::json!({
            "docid": item.id,
            "onlyrecycle": false
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res: Size = Res::parse(&bytes, "Can not get item size")?;
        Ok(res)
    }

    // 文件和文件夹共用命名空间不得重复, 而对于文件的建议名称获取功能更强大一点支持扩展名,
    // 所以就用它替代文件夹的命名算了, 绝大多数情况使用这个, 不过有一种情况会有意外,
    // 对于形如 "Name.Suffix" 的 文件夹, 会得到 "Name (1).Suffix" 这样的建议名称
    /// # Get a suggested name when name conflict in parent directory
    ///
    /// Output:
    ///     - Dir: "[Name] [(Number)]", e.g. "New Folder (1)"
    ///     - File: "[Name] [(Number)].[Suffix]", e.g. "file (1).zip"
    ///
    /// **Note**: For dir named like "[Name].[Suffix]", the suggested name will be "[Name] [(Number)].[Suffix]"
    pub async fn get_suggest_name(&self, parent: &Item, name: &str) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/getsuggestname";
        let json = serde_json::json!({
            "docid": parent.id,
            "name": name,
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res = utils::parse_by_tag(&bytes, "\"name\":\"", "\"")
            .ok_or_else(|| parse_error("Can not get suggest name", &bytes, &"No 'name' field"))?;
        Ok(res.to_string())
    }

    /// # Create directory
    pub async fn create_dir(&self, parent: &Item, name: &str) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/create";
        let json = serde_json::json!({
            "docid": parent.id,
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
    pub async fn rename_item(&self, item: &Item, name: &str) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/rename";
        let json = serde_json::json!({
            "docid": item.id,
            "name": name,
            "ondup": 1
        });
        let payload = Payload::Json(&json);
        self.universal_request(Method::POST, url, &payload).await?;
        Ok(())
    }

    /// # Move an item
    pub async fn move_item(&self, from: &Item, to: &Item) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/move";
        let json = serde_json::json!({
            "destparent": to.id,
            "docid": from.id,
            "ondup": 1
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res = utils::parse_by_tag(&bytes, "\"docid\":\"", "\"")
            .ok_or_else(|| parse_error("Can not move item", &bytes, &"No 'docid' field"))?;
        Ok(res.to_string())
    }

    /// # Copy an item
    pub async fn copy_item(&self, from: &Item, to: &Item) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/copy";
        let json = serde_json::json!({
            "destparent": to.id,
            "docid": from.id,
            "ondup": 1
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res = utils::parse_by_tag(&bytes, "\"docid\":\"", "\"")
            .ok_or_else(|| parse_error("Can not copy item", &bytes, &"No 'docid' field"))?;
        Ok(res.to_string())
    }

    // 文件的接口更干净一点
    // 重复删掉文件也不会报错
    /// # Delete an item to recycle bin
    pub async fn delete_item(&self, item: &Item) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/delete";
        let json = serde_json::json!({
            "docid": item.id,
        });
        let payload = Payload::Json(&json);
        self.universal_request(Method::POST, url, &payload).await?;
        Ok(())
    }

    /// # List recycle bin contents of user's personal directory
    pub async fn list_recycle(&self) -> crate::Result<Dir> {
        let id = self.get_user_dir().await?.id;
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
    /// **Note**: Delete multiple files need call multiple times.
    pub async fn delete_recycle_item(&self, item: &Item) -> crate::Result<()> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/recycle/delete";
        let json = serde_json::json!({
            "docid": item.id,
        });
        let payload = Payload::Json(&json);
        self.universal_request(Method::POST, url, &payload).await?;
        Ok(())
    }

    /// # Restore an item from recycle bin
    pub async fn restore_recycle_item(&self, item: &Item) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/recycle/restore";
        let json = serde_json::json!({
            "docid": item.id,
            "ondup": 1
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;

        let res = utils::parse_by_tag(&bytes, "\"docid\":\"", "\"").ok_or_else(|| {
            parse_error("Can not restore recycle item", &bytes, &"No 'docid' field")
        })?;
        Ok(res.to_string())
    }

    /// # List all share records of the user
    pub async fn share_history(&self) -> crate::Result<Vec<Share>> {
        let url = "https://bhpan.buaa.edu.cn/api/doc-share/v1/docs-shared-with-anyone";
        let payload = Payload::<'_, ()>::Empty;
        let bytes = self.universal_request(Method::GET, url, &payload).await?;
        let res = Share::parse_history(&bytes)?;
        Ok(res)
    }

    // 传入不存在的 id 会在上层触发 400 错误
    /// # Get item share record
    pub async fn share_record(&self, item: &Item) -> crate::Result<Vec<Share>> {
        let url = format!(
            "https://bhpan.buaa.edu.cn/api/shared-link/v1/document/folder/{}?type=anonymous",
            item.id.replace(':', "%3A").replace('/', "%2F")
        );
        let payload = Payload::<'_, ()>::Empty;
        let bytes = self.universal_request(Method::GET, &url, &payload).await?;
        let res = serde_json::from_slice::<Vec<Share>>(&bytes)?;
        Ok(res)
    }

    /// # Share an item
    ///
    /// - Input: [Share] from [Item::to_share()]
    /// - Output: New [Share] with ID field filled
    ///
    /// **Note**: The share link can be formed as `https://bhpan.buaa.edu.cn/link/{ID}`.
    pub async fn share_item(&self, mut share: Share) -> crate::Result<Share> {
        let url = "https://bhpan.buaa.edu.cn/api/shared-link/v1/document/anonymous";
        let payload = Payload::Json(&share);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let res = utils::parse_by_tag(&bytes, "\"id\":\"", "\"")
            .ok_or_else(|| parse_error("Can not create share link", &bytes, &"No 'id' field"))?;
        share.id = res.to_string();
        Ok(share)
    }

    /// # Update share
    ///
    /// - Input: Updated [Share] from `share_record` or `share_item`
    pub async fn share_update(&self, share: &Share) -> crate::Result<()> {
        let url = format!(
            "https://bhpan.buaa.edu.cn/api/shared-link/v1/document/anonymous/{}",
            share.id
        );
        let payload = Payload::Json(&share);
        self.universal_request(Method::PUT, &url, &payload).await?;
        Ok(())
    }

    /// # Delete share
    ///
    /// - Input: [Share] from `share_record` or `share_item`
    pub async fn share_delete(&self, share: &Share) -> crate::Result<()> {
        let url = format!(
            "https://bhpan.buaa.edu.cn/api/shared-link/v1/document/anonymous/{}",
            share.id
        );
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
        let (dirs, files): (Vec<&Item>, Vec<&Item>) = items.iter().partition(|i| i.is_dir());
        let dirs: Vec<&str> = dirs.iter().map(|i| i.id.as_str()).collect();
        let files: Vec<&str> = files.iter().map(|i| i.id.as_str()).collect();

        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/batchdownload";
        let json = serde_json::json!({
            "name": "download.zip",
            "reqhost": "bhpan.buaa.edu.cn",
            "dirs": dirs,
            "files": files
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let mut res = utils::parse_by_tag(&bytes, "\"url\":\"", "\"")
            .ok_or_else(|| parse_error("Can not get download url", &bytes, &"No 'url' field"))?
            .to_string();
        // 服务器返回的 URL 中反斜杠是转义字符, 需要去掉
        res.retain(|c| c != '\\');
        Ok(res)
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

    /// # Upload big file
    ///
    /// - Input: Need call [UploadArgs::compute_mini]
    ///
    /// **Note**: File size should be less than 100 GiB
    #[cfg(feature = "multipart")]
    pub async fn upload_big<R>(&self, args: &UploadArgs, mut reader: R) -> crate::Result<()>
    where
        R: std::io::Read + Send + 'static,
    {
        // 开始上传大文件协议, 获取上传 ID 等
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/osinitmultiupload";
        let json = serde_json::json!({
            "docid": args.dir,
            "length": args.length,
            "name": args.name,
            // 文件名冲突则抛出异常
            "ondup": 1,
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        let init = args.parse_init(&bytes)?;

        // 上传大文件的分块协议, 分块上传文件
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/osuploadpart";
        let json = Payload::Json(&init);
        // 原始数据不保序, 但上传要求严格保序, 且只有最后一个分块大小可以不为 PART_SIZE
        let bytes = self.universal_request(Method::POST, url, &json).await?;
        // 我们在这里预排序
        let part = args.parse_part(&bytes)?;

        let mut part_info = serde_json::Map::<String, Value>::new();
        let mut remaining = args.length;
        for (index, auth) in part {
            // TODO: 难以维护 etag 状态, 暂不支持断点续传
            let key = index.to_string();

            let to_read = std::cmp::min(UploadArgs::PART_SIZE, remaining);
            let mut buffer = vec![0u8; to_read as usize];
            reader
                .read_exact(&mut buffer)
                .map_err(|_| Error::io("Read failed"))?;
            remaining -= to_read;

            let req = self.client.put(&auth[1]);
            // 0 号是方法, 1 号是 URL, 剩余的是 Header
            let req = auth.iter().skip(2).fold(req, |req, header| {
                if let Some((key, value)) = header.split_once(": ") {
                    req.header(key, value)
                } else {
                    req
                }
            });
            let req = req.header("Content-Length", to_read);
            let res = req.body(buffer).send().await?;
            let etag = res
                .headers()
                .get("etag")
                .ok_or_else(|| Error::server("No Etag in Upload").with_label("Cloud"))?
                .as_bytes();
            // 服务器返回的 Etag 只要存在那必然有效. 并且需要去掉双引号
            let etag = String::from_utf8_lossy(&etag);
            let etag = etag.trim_matches('"');
            part_info.insert(key, serde_json::json!([etag, to_read]));
        }

        // 上传大文件的分块完成协议, 提交分块信息
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/oscompleteupload";
        let json = serde_json::json!({
            "docid": init.docid,
            "rev": init.rev,
            "uploadid": init.uploadid,
            "partinfo": part_info,
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(Method::POST, url, &payload).await?;
        // HTTP 方法 POST, 上传链接, 授权 Token, Content-Type, 日期. 和一个 XML Body
        let (complete, body) = UploadArgs::parse_complete(&bytes)?;

        let req = self.client.post(&complete[1]);
        let req = complete.iter().skip(2).fold(req, |req, header| {
            if let Some((key, value)) = header.split_once(": ") {
                req.header(key, value)
            } else {
                req
            }
        });
        // 响应体是 XML 格式的储存桶信息. 无需解析
        // TODO: 日志处理
        let res = req.body(body).send().await?;
        if !res.status().is_success() {
            return Err(Error::server("Complete upload failed")
                .with_label("Cloud")
                .with_source(format!("HTTP status: {}", res.status().as_u16())));
        }

        // Anyshare 我***啊, 都**有大文件分块上传完成协议了
        // 怎么还得单独要这个上传小文件的协议来注册文件, 文档也不写, 排查了半天, 我*了你的*
        // 上传文件完成协议
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/file/osendupload";
        let json = serde_json::json!({
            "docid": init.docid,
            "rev": init.rev,
        });
        let payload = Payload::Json(&json);
        // editor 等无用字段
        let _bytes = self.universal_request(Method::POST, url, &payload).await?;

        Ok(())
    }
}
