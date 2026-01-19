#[cfg(feature = "multipart")]
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Deserializer, Serialize};
use time::macros::format_description;
use time::{PrimitiveDateTime, Weekday};

use crate::{Error, crypto};

/// Request Body Payload
#[derive(Debug, Serialize)]
pub enum Payload<'a, P: Serialize + ?Sized> {
    /// Query data
    Query(&'a P),
    /// JSON data
    Json(&'a P),
}

/// Response Wrapper
#[derive(Deserialize)]
pub struct Res<T> {
    code: u32,
    msg: Option<String>,
    content: T,
}

impl<'de, T: Deserialize<'de>> Res<T> {
    pub(crate) fn parse(v: &'de [u8]) -> crate::Result<T> {
        // TODO: 这种错误怎么处理
        // {"code":0,"msg":"异常错误，错误ID：d0a067d1c90f475381e5654f8847db2e","msg_en":null,"content":"d0a067d1c90f475381e5654f8847db2e"}
        let res: Res<T> = serde_json::from_slice(&v)?;
        // 凭据过期 code 也是 200, 那你这 code 有什么用啊
        if res.code != 200 {
            let source = format!(
                "Status Code: {}. Error Message: {:?}",
                res.code,
                res.msg.unwrap_or_else(|| "No message".to_string())
            );
            return Err(crate::Error::server("Operation failed")
                .with_label("Spoc")
                .with_source(source));
        }
        Ok(res.content)
    }
}

// 辅助容器
pub(super) struct Data<T>(pub T);

// ====================
// 用于 get_week
// ====================

// Res<Week>
/// For `get_week_schedule`, you can get it through `get_week`, and manual builds are generally not recommended
#[derive(Debug, Deserialize)]
pub struct Week {
    /// Week date range
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(rename = "pjmrrq")]
    pub date: (String, String),
    /// Term ID
    #[serde(rename = "mrxq")]
    pub term: String,
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<(String, String), D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let mut s = s.split(",");
    s.next();
    let start = s
        .next()
        .ok_or_else(|| serde::de::Error::custom("Missing start date"))?;
    let end = s
        .next()
        .ok_or_else(|| serde::de::Error::custom("Missing end date"))?;
    Ok((start.to_string(), end.to_string()))
}

// ====================
// 用于 get_week_schedule
// ====================

// Res<Vec<Schedule>>
/// Weekly Schedule item
#[derive(Debug, Deserialize)]
pub struct Schedule {
    /// Course weekday
    #[serde(deserialize_with = "deserialize_weekday")]
    pub weekday: Weekday,
    // 极少数课程可能为空. 那我问你, 提供个空字符串保证结构会死吗
    /// Classroom
    #[serde(default)]
    #[serde(rename = "skdd")]
    pub position: Option<String>,
    /// Teacher
    #[serde(rename = "jsxm")]
    pub teacher: String,
    /// Course name
    #[serde(rename = "kcmc")]
    pub name: String,
    /// Course time range
    #[serde(deserialize_with = "deserialize_time_range")]
    #[serde(rename = "kcsj")]
    pub time: TimeRange,
}

fn deserialize_weekday<'de, D>(deserializer: D) -> Result<Weekday, D::Error>
where
    D: Deserializer<'de>,
{
    let value: String = Deserialize::deserialize(deserializer)?;
    match value.as_str() {
        "monday" => Ok(Weekday::Monday),
        "tuesday" => Ok(Weekday::Tuesday),
        "wednesday" => Ok(Weekday::Wednesday),
        "thursday" => Ok(Weekday::Thursday),
        "friday" => Ok(Weekday::Friday),
        "saturday" => Ok(Weekday::Saturday),
        "sunday" => Ok(Weekday::Sunday),
        _ => Err(serde::de::Error::custom(
            "Unexpected value in SpocSchedule weekday",
        )),
    }
}

/// Course time range
#[derive(Debug)]
pub struct TimeRange {
    /// Course start time
    pub start: PrimitiveDateTime,
    /// Course end time
    pub end: PrimitiveDateTime,
}

fn deserialize_time_range<'de, D>(deserializer: D) -> Result<TimeRange, D::Error>
where
    D: Deserializer<'de>,
{
    let format_string = format_description!("[year]-[month]-[day] [hour]:[minute]");

    let s: String = Deserialize::deserialize(deserializer)?;

    let parts: Vec<&str> = s.split(' ').collect();
    if parts.len() != 2 {
        return Err(serde::de::Error::custom("Invalid time range format"));
    }

    let date_part = parts[0];
    let time_parts: Vec<&str> = parts[1].split('-').collect();
    if time_parts.len() != 2 {
        return Err(serde::de::Error::custom("Invalid time range format"));
    }

    let start_time = format!("{} {}", date_part, time_parts[0]);
    let end_time = format!("{} {}", date_part, time_parts[1]);

    let start =
        PrimitiveDateTime::parse(&start_time, &format_string).map_err(serde::de::Error::custom)?;
    let end =
        PrimitiveDateTime::parse(&end_time, &format_string).map_err(serde::de::Error::custom)?;

    Ok(TimeRange { start, end })
}

// ====================
// 用于 query_courses
// ====================

/// Course item
#[derive(Debug, Deserialize)]
pub struct Course {
    /// Course ID
    #[serde(rename = "kcid")]
    pub id: String,
    /// Course name
    #[serde(rename = "kcmc")]
    pub name: String,
    // // Tearcher name
    // #[serde(rename = "skjs")]
    // pub teacher: String,
}

// ====================
// 用于 query_homeworks
// ====================

impl<'de> Deserialize<'de> for Data<Vec<Homework>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            list: Vec<Homework>,
        }
        let i = I::deserialize(deserializer)?;
        Ok(Data(i.list))
    }
}

/// Homework list item
#[derive(Debug, Deserialize)]
pub struct Homework {
    /// Homework ID
    pub id: String,
    /// Homework title
    #[serde(rename = "zymc")]
    pub title: String,
    // /// Score
    // #[serde(rename = "zyfs")]
    // pub score: u32,
    /// Start datetime
    #[serde(rename = "zykssj")]
    pub start: String,
    /// End datetime
    #[serde(rename = "zyjzsj")]
    pub end: String,
    // 1 为可提交, 0 为不可提交
    /// Status
    #[serde(rename = "sfzysjn")]
    pub status: String,
    /// Course ID
    #[serde(rename = "sskcid")]
    pub course_id: String,
    // 作业属于某周某节课
    // #[serde(rename = "treemlmc")]
    // pub belong: String,
}

// 上面有的字段这里都有, 但没什么用
/// Homework detail
#[derive(Debug, Deserialize)]
pub struct HomeworkDetail {
    /// Homework content (Contain \n)
    #[serde(deserialize_with = "deserialize_homework_content")]
    #[serde(rename = "zynr")]
    pub content: String,
    /// File type
    #[serde(rename = "xzwjlx")]
    pub file: String,
    /// submit times limits
    #[serde(rename = "xztjcs")]
    pub submit_limit: String,
}

fn deserialize_homework_content<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let str: String = Deserialize::deserialize(deserializer)?;
    let str = str.replace("<p>", "").replace("</p>", "");
    Ok(str)
}

/// Upload file arguments
#[derive(Debug, Serialize)]
pub struct UploadArgs {
    #[serde(rename = "chunkNumber")]
    index: usize,
    /// Size of each chunk (fixed)
    #[serde(rename = "chunkSize")]
    chunk_size: usize,
    #[serde(rename = "currentChunkSize")]
    current_chunk_size: usize,
    #[serde(rename = "totalSize")]
    len: usize,
    identifier: String,
    // 不能缺少的字段, 这个名字仅在匹配上传时生效, 可以起到一个重命名的作用
    filename: String,
    // #[serde(rename = "relativePath")]
    // relative_path: String,
    #[serde(rename = "totalChunks")]
    total_chunks: usize,
}

impl UploadArgs {
    /// Create UploadArgs from reader
    pub fn from_reader<R>(mut reader: R, name: String) -> crate::Result<Self>
    where
        R: std::io::Read,
    {
        // 暂时先定死 chunk 大小, 这也是网页端规定的大小
        let chunk_size = 2048000;

        let mut len = 0;
        let mut hasher = crypto::md5::Md5::new();
        let mut buffer = [0u8; 8192];
        loop {
            let n = reader
                .read(&mut buffer)
                .map_err(|_| Error::io("Read Failed"))?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
            len += n;
        }
        let hash = hasher.finalize();
        let identifier = crypto::bytes2hex(&hash);
        // 用满了就用满的
        let current_chunk_size = if len > chunk_size { chunk_size } else { len };
        // 块总数
        let total_chunks = if len % chunk_size == 0 {
            len / chunk_size
        } else {
            len / chunk_size + 1
        };
        Ok(UploadArgs {
            index: 1,
            chunk_size,
            current_chunk_size,
            len,
            identifier,
            filename: name,
            total_chunks,
        })
    }

    #[cfg(feature = "multipart")]
    fn build_form(&self, index: usize, data: Vec<u8>) -> Form {
        let current_chunk_size = data.len();
        Form::new()
            .text("chunkNumber", index.to_string())
            .text("chunkSize", self.chunk_size.to_string())
            .text("currentChunkSize", current_chunk_size.to_string())
            .text("totalSize", self.len.to_string())
            .text("identifier", self.identifier.clone())
            // .text("filename", self.filename.clone())
            // .text("relativePath", self.filename.clone())
            .text("totalChunks", self.total_chunks.to_string())
            .part(
                "file",
                Part::bytes(data)
                    // 这个字段很重要, 但具体名字不重要
                    // 名字取决于 merge 操作, 但它必须是 PDF 文件骗骗文件系统
                    // .file_name(self.filename.clone())
                    .file_name("file.pdf")
                    .mime_str("application/pdf")
                    .unwrap(),
            )
    }

    /// Convert to multiple Form iterator
    #[cfg(feature = "multipart")]
    pub(super) fn chunk_iter<R>(&self, mut reader: R) -> impl Iterator<Item = crate::Result<Form>>
    where
        R: std::io::Read,
    {
        let mut chunk_index = 1;
        let mut remaining = self.len;

        std::iter::from_fn(move || {
            if remaining == 0 {
                return None;
            }

            let to_read = std::cmp::min(self.chunk_size, remaining);
            let mut buffer = vec![0u8; to_read];
            match reader.read_exact(&mut buffer) {
                Ok(()) => {
                    remaining -= to_read;
                    let form = self.build_form(chunk_index, buffer);
                    chunk_index += 1;
                    Some(Ok(form))
                }
                Err(e) => Some(Err(Error::io(format!(
                    "Failed to read chunk {}: {}",
                    chunk_index, e
                )))),
            }
        })
    }

    #[cfg(feature = "multipart")]
    pub(super) fn to_merge(&self) -> MergeArgs<'_> {
        MergeArgs {
            identifier: &self.identifier,
            len: self.len,
            filename: &self.filename,
        }
    }
}

/// Merge file arguments
#[cfg(feature = "multipart")]
#[derive(Debug, Serialize)]
pub(super) struct MergeArgs<'a> {
    identifier: &'a str,
    #[serde(rename = "totalSize")]
    len: usize,
    filename: &'a str,
}

/// Upload file response
#[derive(Debug, Deserialize)]
pub struct UploadRes {
    /// File ID
    pub id: String,
    /// File name
    #[serde(rename = "fileName")]
    pub name: String,
    /// File size
    #[serde(rename = "fileSize")]
    pub size: String,
    /// File MD5
    pub md5: String,
}

impl UploadRes {
    // 怎么这么恶心啊, check 就包一层 data 需要用这个, merge 就不需要
    pub(crate) fn from_json(bytes: &[u8]) -> serde_json::Result<Option<Self>> {
        #[derive(Deserialize)]
        struct I {
            data: Option<UploadRes>,
        }
        let res: I = serde_json::from_slice(bytes)?;
        Ok(res.data)
    }

    /// Convert to download URL
    pub fn as_url(&self) -> String {
        format!(
            "https://spoc.buaa.edu.cn/inco-filesystem/fileManagerSystem/downLoadFile?scjlid={}",
            self.id
        )
    }
}
