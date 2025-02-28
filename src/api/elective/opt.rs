use crate::Error;

use super::ElectiveAPI;
use super::utils::{ElectiveOpt, ElectiveFilter, _ElectiveStatus, _ElectiveRes, ElectiveCourses};

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

        // TODO: 处理搜索值为空的情况

        // 因为学校服务器逆天设计导致 JSON 不合法, 这里需要手动截取其中合法的部分, 去掉重复键

        let last_key = if let Some(i) = text.find("secretVal") {
            i
        } else {
            return Err(Error::APIError("Invalid JSON without secretVal".to_string()));
        };
        let end = if let Some(i) = text[last_key..].find(",") {
            // 不加 1 为了去掉逗号
            last_key + i
        } else {
            return Err(Error::APIError("Invalid JSON".to_string()));
        };
        // 修补缺失的括号
        let fix_json = format!("{}{}", &text[..end], "}]}}");
        let res = serde_json::from_str::<_ElectiveRes>(&fix_json)?;
        Ok(res.data)
    }

    /// 查询已选课程
    pub async fn query_selected(&self) -> Result<(), Error> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/select";

        // 获取 token
        let config = self.config.read().unwrap();
        let token = match &config.elective_token {
            Some(t) => t,
            None => return Err(Error::APIError("No Elective Token".to_string())),
        };

        let res = self.post(url).header("Authorization", token).send().await?;
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

        let res = self.post(url).header("Authorization", token).send().await?;
        let body = res.text().await?;
        println!("{}", body);
        Ok(())
    }

    /// # Select Course
    /// Note that you cannot call the login to update the token before calling this function, otherwise the verification will fail
    pub async fn select_course<'a>(&self, opt: &'a ElectiveOpt<'a>) -> crate::Result<String> {
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
        Ok(text)
    }

    /// # Drop Course
    /// Note that you cannot call the login to update the token before calling this function, otherwise the verification will fail
    pub async fn drop_course<'a>(&self, opt: &'a ElectiveOpt<'a>) -> crate::Result<String> {
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
        Ok(text)
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

        let mut filter = ElectiveFilter::new();
        filter.set_range(ElectiveRange::EXTRA);
        filter.set_key("心理学导论".to_string());

        let res = elective.query_course(&filter).await.unwrap();
        let list = res.data;
        let c = list.get(0).unwrap();
        let opt = ElectiveOpt::from(c);

        let res = elective.select_course(&opt).await.unwrap();
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

        course.query_selected().await.unwrap();

        context.save_cookie("cookie.json");
    }
}
