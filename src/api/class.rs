//! BUAA Smart Classroom (智慧教室) API
//!
//! It is used for class sign-in and class attendance inquiry

use reqwest::Response;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{crypto, utils, Error};

use super::boya::query_course::deserialize_time;

crate::wrap_api!(
    /// BUAA Smart Classroom API Wrapper <br>
    /// Call `class()` on `Context` to get an instance of this struct and call corresponding API on this instance.
    ClassAPI,
    class
);

#[derive(Deserialize)]
struct ClassLogin {
    result: ClassLoginResult,
}

#[derive(Deserialize)]
struct ClassLoginResult {
    id: String,
}

#[derive(Deserialize)]
struct ClassCourses {
    #[serde(deserialize_with = "deserialize_filtered_courses")]
    result: Vec<ClassCourse>,
}

#[cfg_attr(feature = "table", derive(tabled::Tabled))]
#[derive(Debug, Deserialize, Serialize)]
pub struct ClassCourse {
    #[serde(rename = "course_id")]
    pub id: String,
    #[serde(rename = "course_name")]
    pub name: String,
    #[serde(rename = "teacher_name")]
    pub teacher: String,
}

fn deserialize_filtered_courses<'de, D>(deserializer: D) -> Result<Vec<ClassCourse>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut courses: Vec<ClassCourse> = Vec::new();
    let values: Vec<Value> = Deserialize::deserialize(deserializer)?;

    for value in values {
        if let Ok(course) = serde_json::from_value::<ClassCourse>(value.clone()) {
            if !course.teacher.is_empty() {
                courses.push(course);
            }
        }
    }

    Ok(courses)
}

#[derive(Deserialize)]
struct ClassSchedules {
    result: Vec<ClassSchedule>,
}

#[cfg_attr(feature = "table", derive(tabled::Tabled))]
#[derive(Debug, Deserialize)]
pub struct ClassSchedule {
    #[serde(rename = "courseSchedId")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(rename = "classBeginTime")]
    pub time: time::PrimitiveDateTime,
    #[serde(rename = "signStatus")]
    pub state: String,
}

impl ClassAPI {
    /// # Smart Classroom Login
    pub async fn login(&self) -> crate::Result<()> {
        // 获取 JSESSIONID
        let res = self.get("https://iclass.buaa.edu.cn:8346/").send().await?;

        // 整个这一次请求的意义存疑, 但也许是为了验证 loginName 是否有效
        let url = res.url().as_str();
        // 如果获取失败, 说明登录已过期, 则重新登录
        let login_name = match utils::get_value_by_lable(url, "loginName=", "#/") {
            Some(v) => v,
            None => return Err(Error::LoginExpired("SSO Login Expired".to_string())),
        };
        let url = &url[..url.len() - 2];
        // 使用 DES 加密 URL, 这是下一步请求的参数之一
        let url = crypto::des::des_encrypt(url);
        let params = [("method", "html5GetPrivateUserInfo"), ("url", &url)];
        self.get("https://iclass.buaa.edu.cn:8346/wc/auth/html5GetPrivateUserInfo")
            .query(&params)
            .send()
            .await?;

        let params = [
            ("phone", &login_name[..]),
            ("password", ""),
            ("verificationType", "2"),
            ("verificationUrl", ""),
            ("userLevel", "1"),
        ];
        let res = self
            .get("https://iclass.buaa.edu.cn:8346/app/user/login.action")
            .query(&params)
            .send()
            .await?;
        let res = res.text().await?;
        match serde_json::from_str::<ClassLogin>(&res) {
            Ok(res) => {
                let mut config = self.config.write().unwrap();
                config.class_token = Some(res.result.id);
                Ok(())
            }
            Err(_) => Err(Error::LoginError(format!(
                "Smart Classroom Login Failed: {}",
                res
            ))),
        }
    }

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
                ))
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
        let res = serde_json::from_str::<ClassCourses>(&res).unwrap();
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
                ))
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
        let res = serde_json::from_str::<ClassSchedules>(&res)?;
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
                ))
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
    use crate::utils::env;
    use crate::Context;

    #[ignore]
    #[tokio::test]
    async fn test_class_query_course() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);
        context.with_cookies("cookie.json");
        context.login().await.unwrap();

        let class = context.class();
        class.login().await.unwrap();

        let res = class.query_course("202420251").await.unwrap();
        println!("{:#?}", res);

        context.save();
    }

    #[tokio::test]
    async fn test_class_query_schedule() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);
        context.with_cookies("cookie.json");
        context.login().await.unwrap();

        let class = context.class();
        class.login().await.unwrap();

        let res = class.query_schedule("64668").await.unwrap();
        println!("{:#?}", res);

        context.save();
    }

    #[tokio::test]
    async fn test_class_checkin() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);
        context.with_cookies("cookie.json");
        context.login().await.unwrap();

        let class = context.class();
        class.login().await.unwrap();

        let res = class.checkin("2090542").await.unwrap();
        println!("{}", res.text().await.unwrap());

        context.save();
    }
}
