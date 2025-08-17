use super::data::{CloudDir, CloudRootDir};

impl super::CloudApi {
    /// Get directory by type, possible types:
    /// - `""`: All directories
    /// - `"user_doc_lib"`: User's personal directory
    /// - `"shared_user_doc_lib"`: Shared directory
    /// - `"department_doc_lib"`: Department directory
    /// - `"custom_doc_lib"`: Other directory
    pub async fn get_root_dir(&self, r#type: &str) -> crate::Result<Vec<CloudRootDir>> {
        let token = self.token().await?;
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/entry-doc-lib";
        let mut query = vec![("sort", "doc_lib_name"), ("direction", "asc")];
        if !r#type.is_empty() {
            query.push(("type", r#type));
        }
        let res = self
            .get(url)
            .bearer_auth(token)
            .query(&query)
            .send()
            .await?
            .json::<Vec<CloudRootDir>>()
            .await?;
        Ok(res)
    }

    /// Return All Type Root directory
    pub async fn get_all_dir(&self) -> crate::Result<Vec<CloudRootDir>> {
        self.get_root_dir("").await
    }

    /// Return User Root directory ID
    pub async fn get_user_dir_id(&self) -> crate::Result<String> {
        let res = self.get_root_dir("user_doc_lib").await?;
        let id = res
            .into_iter()
            .next()
            .map(|item| item.id)
            .ok_or_else(|| crate::Error::server("[Cloud] No user dir found"))?;
        Ok(id)
    }

    pub async fn list_dir(&self, id: &str) -> crate::Result<CloudDir> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/list";
        let data = serde_json::json!({
            "docid": id,
            "sort": "asc",
            "by": "name"
        });
        let text = self.universal_request(url, &data).await?;
        let res = serde_json::from_str::<CloudDir>(&text)?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[tokio::test]
    async fn test_get_list() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();

        let dir = cloud.get_user_dir_id().await.unwrap();
        let list = cloud.list_dir(&dir).await.unwrap();

        println!("list: {list:?}");
    }
}
