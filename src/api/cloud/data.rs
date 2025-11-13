use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize)]
pub(super) struct Res<T> {
    #[serde(flatten)]
    pub res: Option<T>,
    pub cause: Option<String>,
    pub message: Option<String>,
}

pub(super) enum Body<'a, Q: Serialize + ?Sized> {
    Query(&'a Q),
    Json(&'a Q),
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
}

/// Response for move operation
#[derive(Debug, Deserialize)]
pub(crate) struct MoveRes {
    /// Moved item ID
    #[serde(rename = "docid")]
    pub id: String,
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
