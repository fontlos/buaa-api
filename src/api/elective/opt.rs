use crate::Error;

use super::ElectiveAPI;
use super::utils::{
    _ElectiveOpt, _ElectiveRes1, _ElectiveRes2, _ElectiveStatus, ElectiveCourses, ElectiveFilter,
    ElectiveSeleted,
};

impl ElectiveAPI {
    pub async fn gen_filter(&self) -> crate::Result<ElectiveFilter> {
        let url = "https://byxk.buaa.edu.cn/xsxk/web/studentInfo";

        // 获取 token
        let config = self.config.read().unwrap();
        let token = match &config.elective_token {
            Some(t) => t,
            None => return Err(Error::APIError("No Elective Token".to_string())),
        };

        let query = [("token", token)];

        let res = self
            .post(url)
            .header("Authorization", token)
            .query(&query)
            .send()
            .await?;
        let text = res.text().await?;
        let campus = crate::utils::get_value_by_lable(&text, "\"campus\": \"", "\"");
        if let Some(campus) = campus {
            match campus.parse::<u8>() {
                Ok(campus) => return Ok(ElectiveFilter::new(campus)),
                Err(_) => return Err(Error::APIError("Invalid Campus".to_string())),
            };
        } else {
            return Err(Error::APIError("No Campus".to_string()));
        }
    }
    /// Query Course
    pub async fn query_course(&self, filter: &ElectiveFilter) -> Result<ElectiveCourses, Error> {
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
        let text = res.text().await?;
        let status = serde_json::from_str::<_ElectiveStatus>(&text)?;
        if status.code != 200 {
            return Err(Error::APIError(status.msg));
        }

        let res = serde_json::from_str::<_ElectiveRes1>(&text)?;
        Ok(res.data)
    }

    /// Query Selected Course
    pub async fn query_selected(&self) -> Result<Vec<ElectiveSeleted>, Error> {
        // https://byxk.buaa.edu.cn/xsxk/elective/deselect 查询退选记录的 URL, 操作相同, 但感觉没啥用
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/select";

        // 获取 token
        let config = self.config.read().unwrap();
        let token = match &config.elective_token {
            Some(t) => t,
            None => return Err(Error::APIError("No Elective Token".to_string())),
        };

        let res = self.post(url).header("Authorization", token).send().await?;
        let text = res.text().await?;
        let status = serde_json::from_str::<_ElectiveStatus>(&text)?;
        if status.code != 200 {
            return Err(Error::APIError(status.msg));
        }
        let res = serde_json::from_str::<_ElectiveRes2>(&text)?;
        Ok(res.data)
    }

    /// # Select Course
    /// Note that you cannot call the login to update the token before calling this function, otherwise the verification will fail
    pub async fn select_course<'a>(&self, opt: &'a _ElectiveOpt<'a>) -> crate::Result<()> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/buaa/clazz/add";

        // 获取 token
        let config = self.config.read().unwrap();
        let token = match &config.elective_token {
            Some(t) => t,
            None => return Err(Error::APIError("No Elective Token".to_string())),
        };

        let res = self
            .post(url)
            .form(&opt)
            .header("Authorization", token)
            .send()
            .await?;
        let text = res.text().await?;
        let status = serde_json::from_str::<_ElectiveStatus>(&text)?;
        if status.code != 200 {
            return Err(Error::APIError(status.msg));
        }
        Ok(())
    }

    /// # Drop Course
    /// Note that you cannot call the login to update the token before calling this function, otherwise the verification will fail
    pub async fn drop_course<'a>(&self, opt: &'a _ElectiveOpt<'a>) -> crate::Result<()> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/clazz/del";

        // 获取 token
        let config = self.config.read().unwrap();
        let token = match &config.elective_token {
            Some(t) => t,
            None => return Err(Error::APIError("No Elective Token".to_string())),
        };

        let res = self
            .post(url)
            .form(&opt)
            .header("Authorization", token)
            .send()
            .await?;
        let text = res.text().await?;
        let status = serde_json::from_str::<_ElectiveStatus>(&text)?;
        if status.code != 200 {
            return Err(Error::APIError(status.msg));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::utils::env;

    #[ignore]
    #[tokio::test]
    async fn test_elective_gen_filter() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password).unwrap();
        context.with_cookies("cookie.json").unwrap();
        context.login().await.unwrap();

        let course = context.elective();
        course.login().await.unwrap();

        let _filter = course.gen_filter().await.unwrap();

        context.save_cookie("cookie.json");
    }

    #[ignore]
    #[tokio::test]
    async fn test_elective_query() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password).unwrap();
        context.with_cookies("cookie.json").unwrap();
        context.login().await.unwrap();

        let elective = context.elective();
        elective.login().await.unwrap();

        let filter = elective.gen_filter().await.unwrap();

        let res = elective.query_course(&filter).await.unwrap();
        println!("{:?}", res);

        context.save_cookie("cookie.json");
    }

    #[ignore]
    #[tokio::test]
    async fn test_elective_query_selete() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password).unwrap();
        context.with_cookies("cookie.json").unwrap();
        context.login().await.unwrap();

        let course = context.elective();
        course.login().await.unwrap();

        let res = course.query_selected().await.unwrap();

        println!("{:?}", res);

        context.save_cookie("cookie.json");
    }
}
