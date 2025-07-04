use reqwest::Method;
use serde_json::Value;

use crate::Error;

impl super::CloudAPI {
    pub async fn universal_request(
        &self,
        url: &str,
        data: &Value,
        method: Method,
    ) -> crate::Result<String> {
        // 首先尝试获取 token, 如果没有就可以直接返回了
        let token = match self.cred.load().cloud_token.value() {
            Some(t) => t,
            None => return Err(Error::APIError("No Cloud Token".to_string())),
        };

        let res = match method {
            Method::GET => self.get(url).bearer_auth(token).send().await?,
            Method::POST => self.post(url).bearer_auth(token).json(data).send().await?,
            _ => return Err(Error::APIError("Unsupported Method".to_string())),
        };
        let text = res.text().await?;

        Ok(text)
    }

    pub async fn get_all_dir(&self) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/classified-entry-doc-libs";
        let text = self
            .universal_request(url, &Value::Null, Method::GET)
            .await?;
        Ok(text)
    }

    pub async fn get_user_dir(&self) -> crate::Result<String> {
        let url = "https://bhpan.buaa.edu.cn/api/efast/v1/entry-doc-lib?type=user_doc_lib&sort=doc_lib_name&direction=asc";
        let text = self
            .universal_request(url, &Value::Null, Method::GET)
            .await?;
        // 一个名为 gns 的字段, 代表了文件夹的 ID
        let gns = crate::utils::get_value_by_lable(&text, "gns:\\/\\/", "\"").unwrap();

        Ok(gns.to_string())
    }

    pub async fn list(&self, dir: &str) -> crate::Result<String> {
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
        cloud.login().await.unwrap();

        let dir = cloud.get_user_dir().await.unwrap();
        let list = cloud.list(&dir).await.unwrap();

        println!("list: {list}");
    }
}
