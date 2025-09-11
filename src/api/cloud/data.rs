use serde::{Deserialize, Serialize};

pub(super) enum _CloudBody<'a, Q: Serialize + ?Sized> {
    Query(&'a Q),
    Json(&'a Q),
}

pub enum CloudRoot {
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

impl CloudRoot {
    pub(super) const fn as_query(&self) -> &[(&str, &str)] {
        const SORT: (&str, &str) = ("sort", "doc_lib_name");
        const DIRECTION: (&str, &str) = ("direction", "asc");
        match self {
            CloudRoot::All => &[SORT, DIRECTION],
            CloudRoot::User => &[SORT, DIRECTION, ("type", "user_doc_lib")],
            CloudRoot::Shared => &[SORT, DIRECTION, ("type", "shared_user_doc_lib")],
            CloudRoot::Department => &[SORT, DIRECTION, ("type", "department_doc_lib")],
            CloudRoot::Group => &[SORT, DIRECTION, ("type", "custom_doc_lib")],
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CloudRootDir {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CloudDir {
    pub dirs: Vec<CloudItem>,
    pub files: Vec<CloudItem>,
}

#[derive(Debug, Deserialize)]
pub struct CloudItem {
    #[serde(rename = "create_time")]
    pub create: u64,
    #[serde(rename = "modified")]
    pub modify: u64,
    #[serde(rename = "docid")]
    pub id: String,
    pub name: String,
    #[serde(rename = "rev")]
    pub hash: String,
    // 文件夹大小为 -1
    pub size: i64,
}

impl CloudItem {
    pub fn is_dir(&self) -> bool {
        self.size == -1
    }
}
