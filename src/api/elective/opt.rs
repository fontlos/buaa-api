use serde::Serialize;

use crate::Error;

use super::ElectiveAPI;
use super::utils::{ElectiveCourses, ElectiveFilter, ElectiveSeleted, _ElectiveRes1, _ElectiveRes2, _ElectiveStatus};

impl ElectiveAPI {
    /// 查询课程
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

    /// 查询已选课程
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
    pub async fn select_course<'a, T: Serialize>(&self, opt: &'a T) -> crate::Result<()> {
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
    pub async fn drop_course<'a, T: Serialize>(&self, opt: &'a T) -> crate::Result<()> {
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
    use crate::api::elective::utils::*;
    use crate::utils::env;

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

        let filter = ElectiveFilter::new();

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

        // std::fs::write("data/res2.json", res).unwrap();
        println!("{:?}", res);

        context.save_cookie("cookie.json");
    }
}
