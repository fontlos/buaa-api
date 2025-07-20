use reqwest::Method;
use serde_json::Value;

impl super::CloudAPI {
    pub async fn get_all_dir(&self) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/classified-entry-doc-libs";
        let text = self
            .universal_request(url, &Value::Null, Method::GET)
            .await?;
        Ok(text)
    }

    /// Return User directory ID
    pub async fn get_user_dir(&self) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/entry-doc-lib?type=user_doc_lib&sort=doc_lib_name&direction=asc";
        let text = self
            .universal_request(url, &Value::Null, Method::GET)
            .await?;
        // 一个名为 gns 的字段, 代表了文件夹的 ID
        let gns = crate::utils::get_value_by_lable(&text, "gns:\\/\\/", "\"").unwrap();

        Ok(gns.to_string())
    }

    pub async fn list_dir(&self, dir: &str) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/dir/list";
        let data = serde_json::json!({
            "docid": format!("gns://{dir}"),
            "sort": "asc",
            "by": "name"
        });
        let text = self.universal_request(url, &data, Method::POST).await?;

        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[tokio::test]
    async fn test_get_list() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();

        let dir = cloud.get_user_dir().await.unwrap();
        let list = cloud.list_dir(&dir).await.unwrap();

        println!("list: {list}");
    }
}
