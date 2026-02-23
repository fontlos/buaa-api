use bytes::{BufMut, BytesMut};
use reqwest::Method;

use crate::utils;

use super::{
    Course, Data, Homework, HomeworkDetail, Payload, Res, Schedule, UploadArgs, UploadProgress,
    UploadRes, UploadStatus, Week,
};

impl super::SpocApi {
    /// Get current week
    pub async fn get_week(&self) -> crate::Result<Week> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryOne";
        // SQL ID 是固定值, 应该是对应的数据库键什么的
        let json = serde_json::json!({
            "sqlid": "17275975753144ed8d6fe15425677f752c936d97de1bab76"
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(url, Method::POST, payload).await?;
        let res: Week = Res::parse(&bytes)?;
        Ok(res)
    }

    /// Query schedule of a week
    pub async fn query_week_schedules(&self, week: &Week) -> crate::Result<Vec<Schedule>> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/jxkj/queryRlData";
        let query = [
            ("rllx", "1"), // 日历类型
            ("zksrq", &week.date.0),
            ("zjsrq", &week.date.1),
        ];
        let payload = Payload::Query(&query);
        let bytes = self.universal_request(url, Method::GET, payload).await?;
        let res: Vec<Schedule> = Res::parse(&bytes)?;
        Ok(res)
    }

    /// # Query courses
    ///
    /// `term` format: "yyyy-yyyyt", e.g. "2025-20261" for 2025 fall semester.
    /// Can get from [Week::term]
    pub async fn query_courses(&self, term: &str) -> crate::Result<Vec<Course>> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/jxkj/queryKclb";
        let query = [("xnxq", term)];
        let payload = Payload::Query(&query);
        let bytes = self.universal_request(url, Method::GET, payload).await?;
        let res: Vec<Course> = Res::parse(&bytes)?;
        Ok(res)
    }

    /// Query homeworks
    pub async fn query_homeworks(&self, course: &Course) -> crate::Result<Vec<Homework>> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/kczy/queryXsZyList";
        // 有缓存的情况下没有前两个参数也正常, 但没缓存就会返回 Null
        let query = [("flag", "1"), ("sflx", "2"), ("sskcid", &course.id)];
        let payload = Payload::Query(&query);
        let bytes = self.universal_request(url, Method::GET, payload).await?;
        let res: Data<Vec<Homework>> = Res::parse(&bytes)?;
        Ok(res.0)
    }

    /// Query homework detail
    pub async fn query_homework_detail(&self, hw: &Homework) -> crate::Result<HomeworkDetail> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/kczy/queryKczyInfoByid";
        let query = [("id", &hw.id)];
        let payload = Payload::Query(&query);
        let bytes = self.universal_request(url, Method::GET, payload).await?;
        let res: HomeworkDetail = Res::parse(&bytes)?;
        Ok(res)
    }

    /// Submit homework
    pub async fn submit_homework(&self, hw: &Homework, file: &UploadRes) -> crate::Result<()> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/kczy/submitKcz2";
        // TODO: name 字段真的重要吗, 服务器已经将 ID 与 name 关联在一起了
        let form = [
            ("ytjcs", "2"),
            ("tjfs", "5"),
            ("tjlx", "5"),
            ("sskcid", &hw.course_id),
            ("kczyid", &hw.id),
            ("scwjid_name", &file.name),
            ("scwjid", &file.id),
        ];
        // 原则上是 form, 不过既然能用就不增加复杂度了
        let payload = Payload::Query(&form);
        let bytes = self.universal_request(url, Method::POST, payload).await?;
        // 能写出这种返回值的家里请高人了, msg_en 是给你这么用的吗
        // {"code":200,"msg":"操作成功","msg_en":"操作时间xxx","content":null}
        let _res: Option<()> = Res::parse(&bytes)?;
        Ok(())
    }

    // 查看提交情况, 包括文件 ID 什么的
    // https://spoc.buaa.edu.cn/spocnewht/kczy/queryXsSubmitKczyInfo?kczyid=

    // 上传文件相关
    //
    // 这甚至不需要授权就可以上传下载,
    // 似乎是不限制大小, 类型限制不严格, 常见文件可上传, 对于 DLL, PDB, EXE 等特殊类型可以通过修改后缀或使用压缩包来绕过.
    // 我们在内部实际是使用 PDF 后缀骗骗文件系统. 但依然无法上传特殊后缀的文件也许是为了避免注入攻击
    // 有一个问题是一旦上传出错了, 几乎是不可修改的, 因为 MD5 值已经传上去了, 再上传会触发已经匹配.
    // Check 和 Merge ID 是不同的, 但是对应的文件是相同的
    //
    // 可以通过 `https://spoc.buaa.edu.cn/inco-filesystem/fileManagerSystem/downLoadFile?scjlid=<ID>` 来下载文件.
    // 但这里有个诡异的问题. 文件刚上传不久时, 由于服务器缓存会导致文件分发混乱.
    // 举个例子, 假如上传了两个文件, 获得了 URL1 和 URL2.
    // 首先访问了 URL1, 不去管得到了什么, 然后访问 URL2, 由于 URL2 与 URL1 不同,
    // 触发了缓存, 你会发现你得到了 URL1 对应的文件 (这是什么**逻辑),
    // 而当你第二次访问 URL2 时, 由于 URL2 与上一次相同, 缓存不再生效(?), 你终于得到了正确的文件.
    // 简单来说, 你想要得到正确的文件, 需要访问同一个 URL 两次并丢弃第一次的内容.
    // 这似乎是难以修复的, 因为即使在 Spoc 网页端它们也是通过两次访问同一个 URL 并丢弃第一个来获取正确文件的.
    // 不过好消息是经过几分钟后, 服务器能处理好这种情况.

    /// Upload file with progress callback
    ///
    /// **Note**: Only upload when hash not matched, support resume. And can rename file by same file with new name.
    ///
    /// **Note**: For some special types of files (like DLL, PDB, EXE), the server may reject the upload.
    /// You can try renaming the file with a common extension (like .pdf) or using a compressed archive.
    pub async fn upload_callback<R, F>(
        client: &reqwest::Client,
        args: &UploadArgs,
        reader: R,
        progress: F,
    ) -> crate::Result<UploadRes>
    where
        R: std::io::Read,
        F: Fn(UploadProgress),
    {
        let upload_url = "https://spoc.buaa.edu.cn/inco-filesystem/fileManagerSystem/uploadFile";
        let merge_url = "https://spoc.buaa.edu.cn/inco-filesystem/fileManagerSystem/mergeFile";

        // 检查上传状态, 是否已经存在或者断点续传
        let res = client.get(upload_url).query(args).send().await?;
        let bytes = res.bytes().await?;
        let status = UploadStatus::from_json(&bytes)?;
        match status {
            UploadStatus::Complete(c) => Ok(c),
            UploadStatus::Partial(partial) => {
                let mut done = partial.len() as u64;
                let total = args.total_chunks();

                // 分块上传, 可以乱序, 并且 partial 保序, 如果重复上传会导致合并失败
                // TODO: 并发上传?
                for chunk in args.chunk_iter(reader) {
                    let (index, data) = chunk?;
                    // 已经上传过的跳过
                    if partial.binary_search(&index).is_ok() {
                        continue;
                    }

                    // 可以使用 query 参数, 或者也可以在 multipart/form-data 的 text 字段
                    let size = data.len();
                    let query = [
                        ("chunkNumber", index.to_string()),
                        ("chunkSize", args.chunk_size.to_string()),
                        ("currentChunkSize", size.to_string()),
                        ("totalSize", args.len.to_string()),
                        ("identifier", args.identifier.clone()),
                        // ("filename", args.filename.clone()),
                        // ("relativePath", args.filename.clone()),
                        ("totalChunks", args.total_chunks.to_string()),
                    ];

                    // 由于其他参数可在 query 中传递, 对于纯粹的文件, 我们手动构建这一段 multipart/form-data
                    let boundary = format!("----WebKitFormBoundary{}", utils::gen_rand_str(16));

                    let body_len = size + boundary.len() * 2 + 200; // 200 固定内容, 实际为 111 左右
                    let mut body = BytesMut::with_capacity(body_len);
                    // 起始 boundary: 2 + boundary.len() + 2
                    body.put_slice(b"--");
                    body.put_slice(boundary.as_bytes());
                    body.put_slice(b"\r\n");
                    // Part 头部: 66 + 31
                    // 这个字段很重要, 但具体名字不重要
                    // 名字取决于 merge 操作, 但它必须是 PDF 文件骗骗文件系统
                    body.put_slice(
                        b"Content-Disposition: form-data; name=\"file\"; filename=\"file.pdf\"\r\n",
                    );
                    body.put_slice(b"Content-Type: application/pdf\r\n");
                    // 空行分隔 header 和 body: 2
                    body.put_slice(b"\r\n");
                    // 文件原始二进制数据: size + 2
                    body.put_slice(&data);
                    body.put_slice(b"\r\n");
                    // 结束 boundary: 2 + boundary.len() + 4
                    body.put_slice(b"--");
                    body.put_slice(boundary.as_bytes());
                    body.put_slice(b"--\r\n");

                    let body = body.freeze();

                    // Content-Type
                    let content_type = format!("multipart/form-data; boundary={}", boundary);

                    // 一些无用的 {"data":null,"mc":false}
                    // TODO: 但是为空时也是出错
                    let _res = client
                        .post(upload_url)
                        .query(&query)
                        .header("Content-Type", content_type)
                        .body(body)
                        .send()
                        .await?;

                    done += 1;
                    // 进度回调, 空闭包理应被优化掉
                    progress(UploadProgress { done, total });
                }

                // 合并上传的块
                let query = args.to_merge();
                let res = client.post(merge_url).query(&query).send().await?;
                let bytes = res.bytes().await?;
                // 特殊类型, 如 dll, pdb, exe 等不支持直接 merge, 需要打包成 zip 等上传
                if bytes.is_empty() {
                    return Err(
                        crate::Error::server("Unsupported file type, use '.zip' instead")
                            .with_label("Spoc"),
                    );
                }
                let res = serde_json::from_slice(&bytes)?;

                Ok(res)
            }
        }
    }

    /// # Upload file
    ///
    /// **Note**: Only upload when hash not matched, support resume. And can rename file by same file with new name.
    ///
    /// **Note**: For some special types of files (like DLL, PDB, EXE), the server may reject the upload.
    /// You can try renaming the file with a common extension (like .pdf) or using a compressed archive.
    pub async fn upload<R>(&self, args: &UploadArgs, reader: R) -> crate::Result<UploadRes>
    where
        R: std::io::Read,
    {
        Self::upload_callback(&self.client, args, reader, |_| {}).await
    }
}
