use crate::Error;
use super::ElectiveAPI;

impl ElectiveAPI {
    /// 查询已选课程
    pub async fn query_selected(&self) -> Result<(), Error> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/select";

        // 获取 token
        let config = self.config.read().unwrap();
        let token = match &config.elective_token {
            Some(t) => t,
            None => return Err(Error::APIError("No Elective Token".to_string())),
        };

        let res = self
            .post(url)
            .header("Authorization", token)
            .send()
            .await?;
        let body = res.text().await?;
        println!("{}", body);
        Ok(())
    }

    /// 查询退选记录
    pub async fn query_deselected(&self) -> Result<(), Error> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/deselect";

        // 获取 token
        let config = self.config.read().unwrap();
        let token = match &config.elective_token {
            Some(t) => t,
            None => return Err(Error::APIError("No Elective Token".to_string())),
        };

        let res = self
            .post(url)
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

        course.query_selected().await.unwrap();

        context.save_cookie("cookie.json");
    }
}
