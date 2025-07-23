use serde::Deserialize;

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
    // 空文件夹大小为 -1
    pub size: i64,
}
