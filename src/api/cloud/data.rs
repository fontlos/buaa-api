use serde::{Deserialize, Deserializer, Serialize};

use crate::error::Error;
use crate::utils;

pub(super) enum Payload<'a, P: Serialize + ?Sized> {
    Query(&'a P),
    Json(&'a P),
    Empty,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub(super) struct Res<T>(T);

impl<'de, T: Deserialize<'de>> Res<T> {
    /// 当响应正确时只有目标字段
    pub(super) fn parse(v: &'de [u8], err: &'static str) -> crate::Result<T> {
        let res: Res<T> =
            serde_json::from_slice(&v).map_err(|e| res_error(err, v, Some(e)))?;
        Ok(res.0)
    }
}

/// 尝试从来自服务器的异常响应 JSON 解析错误. 设计好的 API 理应不会再触发这个函数
#[cold]
pub(super) fn res_error(err: &'static str, raw: &[u8], source: Option<impl std::fmt::Display>) -> Error {
    if log::log_enabled!(log::Level::Error) {
        if let Some(source) = source {
            log::error!("Error Source: {}", source);
        }
        // 尝试结构化错误
        // code 字段即使返回错误码, 也没什么查阅 Anyshare 文档的意义, 应该由我自己排查
        // message 字段基本就是 cause 字段的省略版
        // 只有第一句话是有用的, 后面的是服务器内部报错行数与我们无关
        let cause = utils::parse_by_tag(&raw, "\"cause\":\"", "。");
        if let Some(cause) = cause {
            log::info!("Server Cause: {}", cause);
        } else {
            let raw = String::from_utf8_lossy(&raw);
            log::info!("Raw Response: {}", raw);
        }
    }
    Error::server(err).with_label("Cloud")
}

/// Root directory type
pub enum Root {
    /// All directories
    All,
    /// User's personal directory
    User,
    /// Shared directory
    Shared,
    /// Department directory
    Department,
    /// Group directory
    Group,
}

impl Root {
    pub(super) const fn as_query(&self) -> &[(&str, &str)] {
        const SORT: (&str, &str) = ("sort", "doc_lib_name");
        const DIRECTION: (&str, &str) = ("direction", "asc");
        match self {
            Root::All => &[SORT, DIRECTION],
            Root::User => &[SORT, DIRECTION, ("type", "user_doc_lib")],
            Root::Shared => &[SORT, DIRECTION, ("type", "shared_user_doc_lib")],
            Root::Department => &[SORT, DIRECTION, ("type", "department_doc_lib")],
            Root::Group => &[SORT, DIRECTION, ("type", "custom_doc_lib")],
        }
    }
}

/// Root directory info
#[derive(Debug, Deserialize)]
pub struct RootDir {
    /// Root directory ID
    pub id: String,
    /// Root directory name
    pub name: String,
}

/// Directory info
#[derive(Debug, Deserialize)]
pub struct Dir {
    /// Subdirectories
    pub dirs: Vec<Item>,
    /// Files
    pub files: Vec<Item>,
}

/// File or Directory info
#[derive(Debug, Deserialize)]
pub struct Item {
    /// Creation time (timestamp)
    #[serde(rename = "create_time")]
    pub create: u64,
    /// Modification time (timestamp)
    #[serde(rename = "modified")]
    pub modify: u64,
    /// Item ID
    #[serde(rename = "docid")]
    pub id: String,
    /// Item name
    pub name: String,
    /// Item hash
    #[serde(rename = "rev")]
    pub hash: String,
    /// Item size (in bytes, -1 for directories)
    pub size: i64,
}

impl Item {
    /// Check if the item is a directory
    pub fn is_dir(&self) -> bool {
        self.size == -1
    }

    /// For `share_item` method
    pub fn to_share(&self) -> Share {
        let kind = if self.is_dir() { "folder" } else { "file" };
        Share {
            id: None,
            item: ShareItem {
                id: Some(self.id.clone()),
                kind: Some(kind),
                permission: Permission::new(),
            },
            title: self.name.clone(),
            expires_at: "1970-01-01T08:00:00+08:00".to_string(),
            password: "".to_string(),
            limited_times: -1,
        }
    }
}

/// Response for item size
#[derive(Debug, Deserialize)]
pub struct SizeRes {
    /// Number of directories
    #[serde(rename = "dirnum")]
    pub dir: i64,
    /// Number of files
    #[serde(rename = "filenum")]
    pub file: i64,
    /// Item size (in bytes)
    #[serde(rename = "totalsize")]
    pub size: i64,
}

/// Response for create directory
#[derive(Debug, Deserialize)]
pub struct CreateRes {
    /// Creation time (timestamp)
    #[serde(rename = "create_time")]
    pub create: u64,
    /// Modification time (timestamp)
    #[serde(rename = "modified")]
    pub modify: u64,
    /// Item ID
    #[serde(rename = "docid")]
    pub id: String,
    /// Item hash
    #[serde(rename = "rev")]
    pub hash: String,
}

/// Response for move operation
#[derive(Debug, Deserialize)]
pub(crate) struct MoveRes {
    /// Moved item ID
    #[serde(rename = "docid")]
    pub id: String,
}

/// Recycle Directory info
#[derive(Debug, Deserialize)]
pub struct RecycleDir {
    /// Subdirectories
    pub dirs: Vec<RecycleItem>,
    /// Files
    pub files: Vec<RecycleItem>,
}

/// Recycle File or Directory info
#[derive(Debug, Deserialize)]
pub struct RecycleItem {
    /// Modification time (timestamp)
    #[serde(rename = "modified")]
    pub modify: u64,
    /// Item ID
    #[serde(rename = "docid")]
    pub id: String,
    /// Item name
    pub name: String,
    /// Item size (in bytes, -1 for directories)
    pub size: i64,
}

// 得益于服务端垃圾设计
// 共享 API 和共享管理 API 所需的 JSON 相似但又不完全相似
// 导致这整个结构很扭曲
/// Share Item
#[derive(Debug, Deserialize, Serialize)]
pub struct Share {
    // 只在反序列化时用到的字段
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    item: ShareItem,
    title: String,
    expires_at: String,
    password: String,
    limited_times: i64,
}

impl Share {
    /// Set share link title
    pub fn set_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set share link password. Four characters.
    pub fn set_password(mut self, password: &str) -> Self {
        self.password = password.to_string();
        self
    }

    /// Set share link expiration time
    ///
    /// Format: "YYYY-MM-DDTHH:MM:SS+08:00"
    ///
    /// Default is "1970-01-01T08:00:00+08:00" (never expire)
    pub fn set_expire_time(mut self, expire: &str) -> Self {
        self.expires_at = expire.to_string();
        self
    }

    /// Set share link limited times
    pub fn set_limited_times(mut self, times: i64) -> Self {
        self.limited_times = times;
        self
    }

    /// Enable preview permission
    pub fn enable_preview(mut self) -> Self {
        self.item.permission.0 |= Permission::PREVIEW;
        self
    }

    /// Enable download permission
    pub fn enable_download(mut self) -> Self {
        self.item.permission.0 |= Permission::DOWNLOAD;
        self
    }

    /// Enable upload permission (create + modify)
    pub fn enable_upload(mut self) -> Self {
        self.item.permission.0 |= Permission::CREATE | Permission::MODIFY;
        self
    }

    /// Enable modify permission
    pub fn enable_modify(mut self) -> Self {
        self.item.permission.0 |= Permission::MODIFY;
        self
    }

    /// Enable create permission
    pub fn enable_create(mut self) -> Self {
        self.item.permission.0 |= Permission::CREATE;
        self
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ShareItem {
    #[serde(default)]
    #[serde(skip_deserializing)]
    id: Option<String>,
    #[serde(default)]
    #[serde(skip_deserializing)]
    #[serde(rename = "type")]
    kind: Option<&'static str>,
    #[serde(rename = "allow")]
    permission: Permission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Permission(u8);

impl Permission {
    const CREATE: u8 = 0b00001;
    const MODIFY: u8 = 0b00010;
    const DOWNLOAD: u8 = 0b00100;
    const PREVIEW: u8 = 0b01000;
    const DISPLAY: u8 = 0b10000;

    pub const fn new() -> Self {
        Self(Self::DISPLAY)
    }

    fn contains(&self, perm: u8) -> bool {
        (self.0 & perm) != 0
    }
}

impl<'de> Deserialize<'de> for Permission {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<&'de str> = Deserialize::deserialize(deserializer)?;
        let mut perm = Self::new();
        for p in vec {
            match p {
                "create" => perm.0 |= Self::CREATE,
                "modify" => perm.0 |= Self::MODIFY,
                "download" => perm.0 |= Self::DOWNLOAD,
                "preview" => perm.0 |= Self::PREVIEW,
                "display" => perm.0 |= Self::DISPLAY,
                _ => {}
            }
        }
        Ok(perm)
    }
}

impl Serialize for Permission {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        // 预计算数量
        let mut cnt = 0usize;
        if self.contains(Self::DISPLAY) {
            cnt += 1;
        }
        if self.contains(Self::DOWNLOAD) {
            cnt += 1;
        }
        if self.contains(Self::PREVIEW) {
            cnt += 1;
        }
        if self.contains(Self::CREATE) {
            cnt += 1;
        }
        if self.contains(Self::MODIFY) {
            cnt += 1;
        }

        let mut seq = serializer.serialize_seq(Some(cnt))?;
        if self.contains(Self::CREATE) {
            seq.serialize_element("create")?;
        }
        if self.contains(Self::MODIFY) {
            seq.serialize_element("modify")?;
        }
        if self.contains(Self::DOWNLOAD) {
            seq.serialize_element("download")?;
        }
        if self.contains(Self::PREVIEW) {
            seq.serialize_element("preview")?;
        }
        if self.contains(Self::DISPLAY) {
            seq.serialize_element("display")?;
        }
        seq.end()
    }
}

/// Args for real upload
#[derive(Debug, Deserialize)]
pub struct UploadArgs {
    /// Upload request authorization
    #[serde(deserialize_with = "deserialize_upload_args")]
    #[serde(rename = "authrequest")]
    pub auth: UploadAuth,
    /// Uploaded item ID
    #[serde(rename = "docid")]
    pub id: String,
    /// Uploaded item hash
    #[serde(rename = "rev")]
    pub hash: String,
    // name没有解析的必要
}

/// Upload request authorization
#[derive(Debug, Deserialize)]
pub struct UploadAuth {
    /// URL
    pub url: String,
    /// Policy
    pub policy: String,
    /// Signature
    pub signature: String,
    /// Key
    pub key: String,
}

fn deserialize_upload_args<'de, D>(deserializer: D) -> Result<UploadAuth, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Vec<String> = Deserialize::deserialize(deserializer)?;
    if value.len() != 7 {
        return Err(serde::de::Error::custom("Invalid upload args format"));
    }
    let url = value
        .get(1)
        .ok_or_else(|| serde::de::Error::custom("missing field `url`"))?
        .to_string();
    let policy = value
        .get(4)
        .and_then(|s| s.split_once(": ").map(|(_, v)| v))
        .ok_or_else(|| serde::de::Error::custom("missing field `policy`"))?
        .to_string();
    let signature = value
        .get(5)
        .and_then(|s| s.split_once(": ").map(|(_, v)| v))
        .ok_or_else(|| serde::de::Error::custom("missing field `signature`"))?
        .to_string();
    let key = value
        .get(6)
        .and_then(|s| s.split_once(": ").map(|(_, v)| v))
        .ok_or_else(|| serde::de::Error::custom("missing field `key`"))?
        .to_string();
    Ok(UploadAuth {
        url,
        policy,
        signature,
        key,
    })
}
