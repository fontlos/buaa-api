use serde::{Deserialize, Deserializer, Serialize};

use std::fmt::Display;
use std::io::{Read, Seek, SeekFrom};

use crate::crypto::{self, crc, md5};
use crate::error::Error;

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
        let res: Res<T> = serde_json::from_slice(&v).map_err(|e| parse_error(err, v, &e))?;
        Ok(res.0)
    }
}

/// 尝试从来自服务器的异常响应 JSON 解析错误. 设计好的 API 理应不会再触发这个函数
#[cold]
pub(super) fn parse_error(err: &'static str, raw: &[u8], source: &dyn Display) -> Error {
    if log::log_enabled!(log::Level::Error) {
        let raw = String::from_utf8_lossy(&raw);
        log::error!("Error Source: {}", source);
        log::info!("Raw Response: {}", raw);
    }
    Error::parse(err).with_label("Cloud")
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

impl RootDir {
    /// Convert to [Item] with size -1 (indicating a directory)
    pub fn into_item(self) -> Item {
        Item {
            create: 0,
            modify: 0,
            id: self.id,
            name: self.name,
            size: -1,
        }
    }
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
    // 对于回收站, 这个字段不存在
    /// Creation time (timestamp). For recycle item, this field is missing
    #[serde(default)]
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
    // 我们不信任文件版本信息, 理应每次获取最新. 而且这个字段几乎没有用处. 服务器本身并不能返回文件版本列表.
    // 即使使用其来回退或者预览历史版本, 也需要客户端手动缓存版本信息, 而这过于复杂, 需要单个字段仅新增,
    // 而其他字段正常随版本更新而更新
    // 对于回收站, 这个字段不存在
    // Item revision, For recycle item, this field is missing
    // #[serde(default)]
    // pub rev: String,
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
            // 只在反序列化时用到, 所以永远不会被赋值
            id: String::new(),
            item: ShareItem {
                id: Some(self.id.clone()),
                kind: Some(kind),
                permission: Permission::new(),
            },
            title: self.name.clone(),
            expiration: "1970-01-01T08:00:00+08:00".to_string(),
            password: "".to_string(),
            limit: -1,
        }
    }
}

/// Response for item size
#[derive(Debug, Deserialize)]
pub struct Size {
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

// 得益于服务端垃圾设计
// 共享 API 和共享管理 API 所需的 JSON 相似但又不完全相似
// 导致这整个结构很扭曲
/// Share Item
#[derive(Debug, Deserialize, Serialize)]
pub struct Share {
    // 只在反序列化时用到的字段
    /// Share ID. For share link
    #[serde(skip_serializing)]
    pub id: String,
    item: ShareItem,
    /// Share link title
    pub title: String,
    /// Share link expiration time, "1970-01-01T08:00:00+08:00" for never expire
    #[serde(rename = "expires_at")]
    pub expiration: String,
    /// Share link password.
    pub password: String,
    /// Share link limited times, -1 for unlimited
    #[serde(rename = "limited_times")]
    pub limit: i64,
}

impl Share {
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

/// Upload arguments
#[derive(Debug)]
pub struct UploadArgs {
    /// Target directory
    pub dir: String,
    /// File name
    pub name: String,
    /// File length
    pub length: u64,
    /// Slice MD5 (first 200KB)
    pub slice_md5: String,
    /// Full MD5
    pub md5: String,
    /// Full CRC32
    pub crc32: String,
}

impl UploadArgs {
    /// # Create empty UploadArgs
    pub fn new(dir: &Item, name: &str) -> Self {
        Self {
            dir: dir.id.clone(),
            name: name.to_string(),
            length: 0,
            slice_md5: String::new(),
            md5: String::new(),
            crc32: String::new(),
        }
    }

    // 对于普通上传, 尽管不需要校验段 MD5, 这 200KB 的计算空转也无伤大雅
    /// # Create UploadArgs from reader.
    ///
    /// **Note**: CPU intensive task (MD5). But only first 200KiB
    pub fn compute_mini<R>(&mut self, reader: &mut R) -> crate::Result<()>
    where
        R: Read + Seek,
    {
        // 先获取总长度
        let length = reader
            .seek(SeekFrom::End(0))
            .map_err(|_| Error::io("Failed to get length"))?;
        // 回退到文件开头
        reader
            .seek(SeekFrom::Start(0))
            .map_err(|_| Error::io("Failed rewind reader"))?;
        let mut md5_hasher = md5::Md5::new();
        let mut buffer = [0u8; 8192];

        // 取前 200KB 计算检验段 MD5
        const SLICE_SIZE: u64 = 200 * 1024; // 204800 bytes
        let mut len = 0u64;
        loop {
            let n = reader
                .read(&mut buffer)
                .map_err(|_| Error::io("Read Failed"))?;
            if n == 0 {
                break;
            }
            md5_hasher.update(&buffer[..n]);
            len += n as u64;
            if len >= SLICE_SIZE {
                break;
            }
        }
        let md5_hash = md5_hasher.finalize();
        let slice_md5 = crypto::bytes2hex(&md5_hash);
        self.length = length;
        self.slice_md5 = slice_md5;
        // 回退到文件开头
        reader
            .seek(SeekFrom::Start(0))
            .map_err(|_| Error::io("Failed rewind reader"))?;
        Ok(())
    }

    /// # Create UploadArgs from reader.
    ///
    /// **Note**: CPU intensive task (MD5)
    pub fn compute_full<R>(&mut self, reader: &mut R) -> crate::Result<()>
    where
        R: Read + Seek,
    {
        // 先获取总长度
        let length = reader
            .seek(SeekFrom::End(0))
            .map_err(|_| Error::io("Failed to get length"))?;
        // 回退到文件开头
        reader
            .seek(SeekFrom::Start(0))
            .map_err(|_| Error::io("Failed rewind reader"))?;
        let mut md5_hasher = md5::Md5::new();
        let mut crc32_hasher = crc::Crc32::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = reader
                .read(&mut buffer)
                .map_err(|_| Error::io("Read Failed"))?;
            if n == 0 {
                break;
            }
            md5_hasher.update(&buffer[..n]);
            crc32_hasher.update(&buffer[..n]);
        }

        let md5_hash = md5_hasher.finalize();
        let crc32_hash = crc32_hasher.finalize();
        let md5 = crypto::bytes2hex(&md5_hash);
        let crc32 = format!("{:08x}", crc32_hash);
        self.length = length;
        self.md5 = md5;
        self.crc32 = crc32;
        // 回退到文件开头
        reader
            .seek(SeekFrom::Start(0))
            .map_err(|_| Error::io("Failed rewind reader"))?;
        Ok(())
    }

    // 查看分块支持大小 20 MiB - 5 GiB
    // https://bhpan.buaa.edu.cn/api/efast/v1/file/osoption POST EMPTY
    // {"partmaxnum":10000,"partmaxsize":5368709120,"partminsize":20971520}
    // 用 10 MiB 也没报错, 但为了保险起见用 20 MiB
    pub(super) const PART_SIZE: u64 = 20 * 1024 * 1024;

    pub(super) fn parse_init(&self, bytes: &[u8]) -> crate::Result<UploadInit> {
        let part_num = self.length.div_ceil(Self::PART_SIZE);
        let mut init: UploadInit = serde_json::from_slice(&bytes)
            .map_err(|e| parse_error("Can not initialize upload", bytes, &e))?;
        init.parts = format!("1-{}", part_num);
        Ok(init)
    }

    pub(super) fn parse_part(&self, bytes: &[u8]) -> crate::Result<Vec<(u32, [String; 5])>> {
        use std::collections::BTreeMap;
        /// 上传大文件的分块协议
        #[derive(Debug, Deserialize)]
        struct UploadPart {
            // 每个分块的: HTTP 方法 PUT, 上传链接, 授权 Token, Content-Type, 日期
            authrequests: BTreeMap<String, [String; 5]>,
        }
        let part_num = self.length.div_ceil(Self::PART_SIZE) as usize;
        let part: UploadPart = serde_json::from_slice(&bytes)
            .map_err(|e| parse_error("Can not get upload part auth", bytes, &e))?;
        let mut parts = Vec::with_capacity(part_num);
        for (k, v) in part.authrequests {
            // 原则上这必将成功
            if let Ok(num) = k.parse::<u32>() {
                parts.push((num, v));
            }
        }
        // 键无重复
        parts.sort_unstable_by_key(|(num, _)| *num);
        Ok(parts)
    }

    /// 解析上传大文件的分块完成协议参数
    /// 响应体为 multipart/form-data. 一个 XML 作为 Body, 一个 JSON 包含请求参数, 手动解析.
    pub(super) fn parse_complete(bytes: &[u8]) -> crate::Result<([String; 5], Vec<u8>)> {
        let form_err = || Error::server("Bad complete upload response").with_label("Cloud");
        // 查找 XML 开始位置, 搜索 '<'
        let xml_start = bytes.iter().position(|&b| b == b'<').ok_or_else(form_err)?;
        // 继续查找 XML 结束位置, 搜索 '-'
        let xml_end = bytes[xml_start..]
            .iter()
            .position(|&b| b == b'-')
            .ok_or_else(form_err)?;
        let xml_bytes = &bytes[xml_start..xml_start + xml_end];
        // 继续查找 JSON 位置, 搜索 '{'
        let json_start = bytes.iter().position(|&b| b == b'{').ok_or_else(form_err)?;
        // 继续查找 JSON 结束位置, 搜索 `--`
        let json_end = bytes[json_start..]
            .windows(2)
            .position(|w| w == b"--")
            .ok_or_else(form_err)?;
        let json_bytes = &bytes[json_start..json_start + json_end];
        #[derive(Debug, Deserialize)]
        struct UploadComplete {
            // HTTP 方法 POST, 上传链接, 授权 Token, Content-Type, 日期
            authrequest: [String; 5],
        }
        let complete: UploadComplete = serde_json::from_slice(&json_bytes)
            .map_err(|e| parse_error("Can not get complete upload auth", json_bytes, &e))?;
        Ok((complete.authrequest, xml_bytes.to_vec()))
    }
}

// 用于小文件
/// 开始上传文件协议
#[derive(Debug, Deserialize)]
pub(super) struct UploadAuth {
    pub authrequest: Vec<String>,
    pub docid: String,
    pub rev: String,
}

// 用于大文件
/// 开始上传大文件协议
#[derive(Debug, Deserialize, Serialize)]
pub(super) struct UploadInit {
    pub docid: String,
    pub rev: String,
    pub uploadid: String,
    #[serde(skip_deserializing)]
    pub parts: String,
}
