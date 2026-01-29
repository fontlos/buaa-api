use reqwest::Method;
#[cfg(feature = "multipart")]
use tokio::sync::{mpsc, oneshot};

use super::{
    Course, Data, Homework, HomeworkDetail, Payload, Res, Schedule, UploadArgs, UploadRes, Week,
};
#[cfg(feature = "multipart")]
use super::{UploadHandle, UploadProgress};

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

    /// # Upload fast
    ///
    /// **Note**: If hash match, returns `Some(UploadRes)`, otherwise returns `None`.
    /// You need [super::SpocApi::upload_file()] or [super::SpocApi::upload_progress()] for final upload
    /// which need enable `multipart` feature.
    ///
    /// **Note**: This can rename files by same file but with new name.
    pub async fn upload_fast(&self, args: &UploadArgs) -> crate::Result<Option<UploadRes>> {
        let url = "https://spoc.buaa.edu.cn/inco-filesystem/fileManagerSystem/uploadFile";
        let res = self.client.get(url).query(args).send().await?;
        let bytes = res.bytes().await?;
        // 注意这里有一层 data 包裹
        // TODO: 断点续传时可能返回 {"data":[1],"mc":false}
        let res = UploadRes::for_fast(&bytes)?;
        Ok(res)
    }

    /// Internal upload function
    #[cfg(feature = "multipart")]
    async fn upload_internal<R, F>(
        client: &reqwest::Client,
        args: &UploadArgs,
        data: R,
        progress: F,
    ) -> crate::Result<UploadRes>
    where
        R: std::io::Read,
        F: Fn(UploadProgress),
    {
        let upload_url = "https://spoc.buaa.edu.cn/inco-filesystem/fileManagerSystem/uploadFile";
        let merge_url = "https://spoc.buaa.edu.cn/inco-filesystem/fileManagerSystem/mergeFile";

        let mut done = 0;
        let total = args.total_chunks();

        // 分块上传
        for form in args.chunk_iter(data) {
            let form = form?;
            // 一些无用的 {"data":null,"mc":false}
            // TODO: 但是为空时也是出错
            let _res = client.post(upload_url).multipart(form).send().await?;

            done += 1;
            progress(UploadProgress { done, total });
        }

        // 合并上传的块
        let query = args.to_merge();
        let res = client.post(merge_url).query(&query).send().await?;
        let bytes = res.bytes().await?;
        let res = UploadRes::for_merge(&bytes)?;

        Ok(res)
    }

    /// # Upload file
    ///
    /// The real upload when hash not match, need enable `multipart` feature.
    ///
    /// **Note**: This function only upload and merge the file, does not check hash match.
    /// Use [super::SpocApi::upload_fast()] first. Otherwise, the upload is invalid.
    ///
    /// **Note**: For some special types of files (like DLL, PDB, EXE), the server may reject the upload.
    /// You can try renaming the file with a common extension (like .pdf) or using a compressed archive.
    #[cfg(feature = "multipart")]
    pub async fn upload_file<R>(&self, args: &UploadArgs, data: R) -> crate::Result<UploadRes>
    where
        R: std::io::Read,
    {
        Self::upload_internal(&self.client, args, data, |_| {}).await
    }

    /// # Upload file with progress
    ///
    /// The real upload when hash not match, need enable `multipart` feature.
    ///
    /// **Note**: This function only upload and merge the file, does not check hash match.
    /// Use [super::SpocApi::upload_fast()] first. Otherwise, the upload is invalid.
    ///
    /// **Note**: For some special types of files (like DLL, PDB, EXE), the server may reject the upload.
    /// You can try renaming the file with a common extension (like .pdf) or using a compressed archive.
    #[cfg(feature = "multipart")]
    pub fn upload_progress<R>(&self, args: UploadArgs, data: R) -> UploadHandle
    where
        R: std::io::Read + Send + 'static,
    {
        let (progress_tx, progress_rx) = mpsc::unbounded_channel();
        let (result_tx, result_rx) = oneshot::channel();
        let client = self.client.clone();

        let upload = async move {
            Self::upload_internal(&client, &args, data, |progress| {
                let _ = progress_tx.send(progress);
            }).await
        };

        tokio::spawn(async move {
            // crate::Result<UploadRes>
            let result = upload.await;
            let _ = result_tx.send(result);
        });

        UploadHandle {
            result_rx,
            progress_rx,
        }
    }

    /// # An easy Upload
    ///
    /// Combined function of [super::SpocApi::upload_fast()] and [super::SpocApi::upload_file()].
    ///
    /// **Note**: For some special types of files (like DLL, PDB, EXE), the server may reject the upload.
    /// You can try renaming the file with a common extension (like .pdf) or using a compressed archive.
    #[cfg(feature = "multipart")]
    pub async fn upload<R, N>(&self, mut reader: R, name: N) -> crate::Result<UploadRes>
    where
        R: std::io::Read + std::io::Seek + Send + 'static,
        N: Into<String>,
    {
        let name = name.into();

        let (args, reader) = tokio::task::spawn_blocking(move || {
            let start = reader
                .stream_position()
                .map_err(|_| crate::Error::io("Failed to get stream position"))?;
            let args = UploadArgs::from_reader(&mut reader, name)?;
            reader
                .seek(std::io::SeekFrom::Start(start))
                .map_err(|_| crate::Error::io("Failed to seek back"))?;
            crate::Result::Ok((args, reader))
        })
        .await
        .map_err(|e| crate::Error::io("Task error").with_source(e))??;

        // 检查是否匹配
        let res = self.upload_fast(&args).await?;
        if let Some(res) = res {
            return Ok(res);
        }

        // 不存在则分块上传
        self.upload_file(&args, reader).await
    }
}
