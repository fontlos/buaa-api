use super::BoyaAPI;
use super::{_BoyaCourses, BoyaCourse};

impl BoyaAPI {
    /// # Query Course
    pub async fn query_course(&self) -> crate::Result<Vec<BoyaCourse>> {
        let query = "{\"pageNumber\":1,\"pageSize\":10}";
        let url = "https://bykc.buaa.edu.cn/sscv/queryStudentSemesterCourseByPage";
        let res = self.universal_request(query, url).await?;
        let res = serde_json::from_str::<_BoyaCourses>(&res)?;
        Ok(res.data)
    }

    pub async fn query_course_vpn(&self) -> crate::Result<Vec<BoyaCourse>> {
        let query = "{\"pageNumber\":1,\"pageSize\":10}";
        let url = "https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryStudentSemesterCourseByPage";
        let res = self.universal_request(query, url).await?;
        let res = serde_json::from_str::<_BoyaCourses>(&res)?;
        Ok(res.data)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::utils::env;

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_course() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password).unwrap();
        context.with_cookies("cookie.json").unwrap();
        context.login().await.unwrap();

        let boya = context.boya();
        boya.login().await.unwrap();

        let res = match boya.query_course().await {
            Ok(s) => s,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };
        println!("{:?}", res);

        context.save_cookie("cookie.json");
    }
}
