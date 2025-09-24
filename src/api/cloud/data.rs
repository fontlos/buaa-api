use serde::{Deserialize, Serialize};

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
