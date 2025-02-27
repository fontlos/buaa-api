use crate::Error;
use super::ElectiveAPI;
use super::filter::ElectiveFilter;

impl ElectiveAPI {
    /// 查询课程
    pub async fn query_course(&self, filter: &ElectiveFilter) -> Result<(), Error> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/buaa/clazz/list";

        // 获取 token
        let config = self.config.read().unwrap();
        let token = match &config.elective_token {
            Some(t) => t,
            None => return Err(Error::APIError("No Elective Token".to_string())),
        };

        let res = self
            .post(url)
            .json(&filter)
            .header("Authorization", token)
            .send()
            .await?;
        let body = res.text().await?;
        println!("{}", body);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::env;
    use crate::Context;
    use crate::api::elective::filter::ElectiveFilter;

    #[ignore]
    #[tokio::test]
    async fn test_elective() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password).unwrap();
        context.with_cookies("cookie.json").unwrap();
        context.login().await.unwrap();

        let course = context.elective();
        course.login().await.unwrap();

        let filter = ElectiveFilter::new();

        course.query_course(&filter).await.unwrap();
        // let json = serde_json::to_string(&filter).unwrap();
        // println!("{}", json);

        context.save_cookie("cookie.json");
    }
}
