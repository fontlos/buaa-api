use reqwest::Response;

use crate::{Error, utils};

use super::{_ClassCourses, _ClassSchedules, ClassCourse, ClassSchedule};

impl super::ClassAPI {
    /// # Smart Classroom query all course of a term
    /// - Input: Term ID
    ///     - Example: `202320242`` is 2024 spring term, `202420251` is 2024 autumn term
    pub async fn query_course(&self, id: &str) -> crate::Result<Vec<ClassCourse>> {
        let config = self.config.read().unwrap();
        let token = match &config.class_token {
            Some(t) => t,
            None => {
                return Err(Error::LoginError(
                    "Smart Classroom Login Required".to_string(),
                ));
            }
        };
        let res = self.post(
            format!(
                    "https://iclass.buaa.edu.cn:8346/app/choosecourse/get_myall_course.action?user_type=1&id={}&xq_code={}",
                    token,
                    id
                )
            )
            .send()
            .await?;
        let res = res.text().await?;
        let res = serde_json::from_str::<_ClassCourses>(&res).unwrap();
        Ok(res.result)
    }

    /// # Smart Classroom query one course's all schedule
    /// - Input: Course ID, from [ClassCourse]
    pub async fn query_schedule(&self, id: &str) -> crate::Result<Vec<ClassSchedule>> {
        let config = self.config.read().unwrap();
        let token = match &config.class_token {
            Some(t) => t,
            None => {
                return Err(Error::LoginError(
                    "Smart Classroom Login Required".to_string(),
                ));
            }
        };
        let res = self.post(
            format!(
                    "https://iclass.buaa.edu.cn:8346/app/my/get_my_course_sign_detail.action?id={}&courseId={}",
                    token,
                    id
                )
            )
            .send()
            .await?;
        let res = res.text().await?;
        let res = serde_json::from_str::<_ClassSchedules>(&res)?;
        Ok(res.result)
    }

    /// # Smart Classroom checkin schedule
    /// - Input: Schedule ID, from [ClassSchedule]
    pub async fn checkin(&self, id: &str) -> crate::Result<Response> {
        let config = self.config.read().unwrap();
        let token = match &config.class_token {
            Some(t) => t,
            None => {
                return Err(Error::LoginError(
                    "Smart Classroom Login Required".to_string(),
                ));
            }
        };
        let time = utils::get_time();
        let res = self.post(
            format!(
                    "http://iclass.buaa.edu.cn:8081/app/course/stu_scan_sign.action?courseSchedId={}&timestamp={}&id={}",
                    id,
                    time,
                    token
                )
            )
            .send()
            .await?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::utils::env;

    #[ignore]
    #[tokio::test]
    async fn test_class_query_course() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password).unwrap();
        context.with_cookies("cookie.json").unwrap();
        context.login().await.unwrap();

        let class = context.class();
        class.login().await.unwrap();

        let res = class.query_course("202420251").await.unwrap();
        println!("{:#?}", res);

        context.save_cookie("cookie.json");
    }

    #[tokio::test]
    async fn test_class_query_schedule() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password).unwrap();
        context.with_cookies("cookie.json").unwrap();
        context.login().await.unwrap();

        let class = context.class();
        class.login().await.unwrap();

        let res = class.query_schedule("64668").await.unwrap();
        println!("{:#?}", res);

        context.save_cookie("cookie.json");
    }

    #[tokio::test]
    async fn test_class_checkin() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password).unwrap();
        context.with_cookies("cookie.json").unwrap();
        context.login().await.unwrap();

        let class = context.class();
        class.login().await.unwrap();

        let res = class.checkin("2090542").await.unwrap();
        println!("{}", res.text().await.unwrap());

        context.save_cookie("cookie.json");
    }
}
