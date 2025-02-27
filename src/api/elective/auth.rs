use super::ElectiveAPI;
use crate::Error;

impl ElectiveAPI {
    /// # 本研选课登录
    pub async fn login(&self) -> crate::Result<()> {
        let url = "https://sso.buaa.edu.cn/login?service=https%3A%2F%2Fbyxk.buaa.edu.cn%2Fxsxk%2Fauth%2Fcas";
        // 获取 JSESSIONID
        let res = self.get(url).send().await?;
        // 未转跳就证明登录过期
        if res.url().as_str() == url {
            return Err(Error::LoginExpired("SSO Expired".to_string()));
        }
        // 储存 token
        let cookie = self.cookies.lock().unwrap();
        match cookie.get("byxk.buaa.edu.cn", "/xsxk", "token") {
            Some(t) => {
                let mut config = self.config.write().unwrap();
                config.elective_token = Some(t.value().to_string());
                return Ok(());
            }
            None => return Err(Error::LoginError("No Token".to_string())),
        }
    }

    #[deprecated]
    pub async fn get_student_info(&self) -> crate::Result<()> {
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
        let body = res.text().await?;
        println!("{}", body);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::utils::env;

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

        #[allow(deprecated)]
        course.get_student_info().await.unwrap();

        context.save_cookie("cookie.json");
    }
}
