use reqwest::Method;

#[cfg(feature = "multipart")]
use crate::{Error, utils};

use super::{Course, Data, Homework, HomeworkDetail, Payload, Res, Schedule, Week};

#[cfg(feature = "multipart")]
use super::{UploadArgs, UploadRes};

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

    // 查看提交情况, 包括文件 ID 什么的
    // https://spoc.buaa.edu.cn/spocnewht/kczy/queryXsSubmitKczyInfo?kczyid=

    // 似乎是不限制大小, 不限制类型. 我们可以在内部用 PDF 后缀骗骗文件系统
    //
    // 有一个问题是一旦上传出错了, 几乎是不可修改的, 因为 MD5 值已经传上去了, 再上传会触发已经匹配
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
    //
    // Check 和 Merge ID 是不同的, 但是对应的文件是相同的
    //
    /// # Upload file
    ///
    /// **Note**: When file hash not match, `data()` will be called for the second time,
    /// so it should return a full reader of the file to upload.
    #[cfg(feature = "multipart")]
    pub async fn upload<D, R, N>(&self, data: D, name: N) -> crate::Result<UploadRes>
    where
        D: Fn() -> R + Send + Sync + Clone + 'static,
        R: std::io::Read,
        N: Into<String>,
    {
        let data_for_hash = data.clone();
        let name = name.into();
        let args = utils::blocking_compute(move || {
            let reader = data_for_hash();
            UploadArgs::from_reader(reader, name)
        })
        .await
        .map_err(|_| Error::server("Oneshot canceled"))??;

        // 检查是否匹配
        let url = "https://spoc.buaa.edu.cn/inco-filesystem/fileManagerSystem/uploadFile";
        let payload = Payload::Query(&args);
        let res_bytes = self.universal_request(url, Method::GET, payload).await?;
        let id = crate::utils::parse_by_tag(&res_bytes, "id\":\"", "\"");
        if let Some(id) = id {
            let res = UploadRes {
                id: id.to_string(),
                name: args.filename,
            };
            return Ok(res);
        }

        // 不存在则分块上传
        for form in args.chunk_iter(data()) {
            // 一些无用的 {"data":null,"mc":false}
            // TODO: 但是为空时也是出错
            let _ = self.client.post(url).multipart(form?).send().await?;
        }

        // 合并上传的块
        let url = "https://spoc.buaa.edu.cn/inco-filesystem/fileManagerSystem/mergeFile";
        let form = args.to_merge(&args.filename);
        let payload = Payload::Query(&form);
        let res_bytes = self.universal_request(url, Method::POST, payload).await?;
        crate::utils::parse_by_tag(&res_bytes, "id\":\"", "\"")
            .map(|s| UploadRes {
                id: s.to_string(),
                name: args.filename,
            })
            .ok_or_else(|| Error::server("No file id").with_label("Spoc"))
    }
}
