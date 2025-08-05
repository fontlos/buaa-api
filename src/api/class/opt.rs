use reqwest::Response;

use crate::{Error, utils};

use super::{_ClassCourses, _ClassSchedules, ClassCourse, ClassSchedule};

impl super::ClassApi {
    /// # Smart Classroom query all course of a term
    /// - Input: Term ID
    ///     - Example: `202320242`` is 2024 spring term, `202420251` is 2024 autumn term
    pub async fn query_course(&self, id: &str) -> crate::Result<Vec<ClassCourse>> {
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && self.cred.load().class_token.is_expired() {
            self.login().await?;
        }
        let cred = self.cred.load();
        let token = match cred.class_token.value() {
            Some(t) => t,
            None => {
                return Err(Error::AuthError(
                    "No Class Token".to_string(),
                ));
            }
        };
        let res = self.post(
            format!("https://iclass.buaa.edu.cn:8346/app/choosecourse/get_myall_course.action?user_type=1&id={token}&xq_code={id}")
            )
            .send()
            .await?;
        let res = res.text().await?;
        let res = serde_json::from_str::<_ClassCourses>(&res)?;
        Ok(res.result)
    }

    /// # Smart Classroom query one course's all schedule
    /// - Input: Course ID, from [ClassCourse]
    pub async fn query_schedule(&self, id: &str) -> crate::Result<Vec<ClassSchedule>> {
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && self.cred.load().class_token.is_expired() {
            self.login().await?;
        }
        let cred = self.cred.load();
        let token = match cred.class_token.value() {
            Some(t) => t,
            None => {
                return Err(Error::AuthError(
                    "No Class Token".to_string(),
                ));
            }
        };
        let res = self.post(
            format!("https://iclass.buaa.edu.cn:8346/app/my/get_my_course_sign_detail.action?id={token}&courseId={id}")
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
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && self.cred.load().class_token.is_expired() {
            self.login().await?;
        }
        let cred = self.cred.load();
        let token = match cred.class_token.value() {
            Some(t) => t,
            None => {
                return Err(Error::AuthError(
                    "No Class Token".to_string(),
                ));
            }
        };
        let time = utils::get_time_millis();
        let res = self.post(
            format!("http://iclass.buaa.edu.cn:8081/app/course/stu_scan_sign.action?courseSchedId={id}&timestamp={time}&id={token}")
            )
            .send()
            .await?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[ignore]
    #[tokio::test]
    async fn test_class_query_course() {
        let context = Context::with_auth("./data");

        let class = context.class();
        // class.login().await.unwrap();

        let res = class.query_course("202420252").await.unwrap();
        println!("{:#?}", res);

        // context.save_auth("./data");
    }

    #[tokio::test]
    async fn test_class_query_schedule() {
        let context = Context::with_auth("./data");

        let class = context.class();
        class.login().await.unwrap();

        let res = class.query_schedule("64668").await.unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    async fn test_class_checkin() {
        let context = Context::with_auth("./data");

        let class = context.class();
        class.login().await.unwrap();

        let res = class.checkin("2090542").await.unwrap();
        println!("{}", res.text().await.unwrap());
    }
}
