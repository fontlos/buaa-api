//! Smart Classroom System (iclass) API

use reqwest::Response;
use serde::Deserialize;

use crate::{crypto, utils, Session, SessionError};

#[derive(Deserialize)]
struct IClassLogin {
    result: IClassLoginResult,
}

#[derive(Deserialize)]
struct IClassLoginResult {
    id: String,
}

#[derive(Deserialize)]
struct IClassCourses {
    result: Vec<IClassCourse>,
}

#[cfg_attr(feature = "table", derive(tabled::Tabled))]
#[derive(Debug, Deserialize)]
pub struct IClassCourse {
    #[serde(rename = "course_id")]
    pub id: String,
    #[serde(rename = "course_name")]
    pub name: String,
}

#[derive(Deserialize)]
struct IClassSchedules {
    result: Vec<IClassSchedule>,
}

#[cfg_attr(feature = "table", derive(tabled::Tabled))]
#[derive(Debug, Deserialize)]
pub struct IClassSchedule {
    #[serde(rename = "courseSchedId")]
    pub id: String,
    #[serde(rename = "teachTime")]
    pub time: String,
    #[serde(rename = "signStatus")]
    pub state: String,
}

impl Session {
    /// # Smart Classroom Login
    /// - Need: [`sso_login`](#method.sso_login)
    /// - Output: User ID
    pub async fn iclass_login(&self) -> Result<String, SessionError> {
        // 获取 JSESSIONID
        let res = self.get("https://iclass.buaa.edu.cn:8346/").send().await?;

        // 整个这一次请求的意义存疑, 但也许是为了验证 loginName 是否有效
        let url = res.url().as_str();
        // TODO 如果获取失败, 说明登录已过期, 则重新登录
        let login_name = match utils::get_value_by_lable(url, "loginName=", "#/") {
            Some(v) => v,
            None => return Err(SessionError::LoginExpired),
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
        match serde_json::from_str::<IClassLogin>(&res) {
            Ok(res) => Ok(res.result.id),
            Err(_) => Err(SessionError::LoginError(format!(
                "Smart Classroom Login Failed: {}",
                res
            ))),
        }
    }

    /// # Smart Classroom query all course of a term
    /// - Need: [`iclass_login`](#method.iclass_login)
    /// - Input:
    ///     - Term ID
    ///         - Example: `202320242`` is 2024 spring term, `202420251` is 2024 autumn term
    ///     - User ID from [`iclass_login`](#method.iclass_login)
    pub async fn iclass_query_course(
        &self,
        term_id: &str,
        user_id: &str,
    ) -> Result<Vec<IClassCourse>, SessionError> {
        let res = self.post(
            format!(
                    "https://iclass.buaa.edu.cn:8346/app/choosecourse/get_myall_course.action?user_type=1&id={}&xq_code={}",
                    user_id,
                    term_id
                )
            )
            .send()
            .await?;
        let res = res.text().await?;
        let res = serde_json::from_str::<IClassCourses>(&res).unwrap();
        Ok(res.result)
    }

    /// # Smart Classroom query one course's all schedule
    /// - Need:
    ///     - [`iclass_login`](#method.iclass_login)
    ///     - [`iclass_query_course`](#method.iclass_query_course)
    /// - Input:
    ///     - Course ID, from [IClassCourse]
    ///     - User ID from [`iclass_login`](#method.iclass_login)
    pub async fn iclass_query_schedule(
        &self,
        course_id: &str,
        user_id: &str,
    ) -> Result<Vec<IClassSchedule>, SessionError> {
        let res = self.post(
            format!(
                    "https://iclass.buaa.edu.cn:8346/app/my/get_my_course_sign_detail.action?id={}&courseId={}",
                    user_id,
                    course_id
                )
            )
            .send()
            .await?;
        let res = res.text().await?;
        let res = serde_json::from_str::<IClassSchedules>(&res).unwrap();
        Ok(res.result)
    }

    /// # Smart Classroom checkin schedule
    /// - Need:
    ///     - [`iclass_login`](#method.iclass_login)
    ///     - [`iclass_query_schedule`](#method.iclass_query_schedule)
    /// - Input:
    ///     - Schedule ID, from [IClassSchedule]
    ///     - User ID from [`iclass_login`](#method.iclass_login)
    pub async fn iclass_checkin_schedule(
        &self,
        sche_id: &str,
        user_id: &str,
    ) -> Result<Response, SessionError> {
        let time = utils::get_time();
        let res = self.post(
            format!(
                    "http://iclass.buaa.edu.cn:8081/app/course/stu_scan_sign.action?courseSchedId={}&timestamp={}&id={}",
                    sche_id,
                    time,
                    user_id
                )
            )
            .send()
            .await?;
        Ok(res)
    }
}

#[tokio::test]
async fn test_iclass_login() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();

    let user_id = session.iclass_login().await.unwrap();
    println!("User id: {}", user_id);

    session.save();
}

#[tokio::test]
async fn test_iclass_query_course() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();
    let user_id = session.iclass_login().await.unwrap();

    let res = session
        .iclass_query_course("202420251", &user_id)
        .await
        .unwrap();
    println!("{:#?}", res);

    session.save();
}

#[tokio::test]
async fn test_iclass_query_schedule() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();
    let user_id = session.iclass_login().await.unwrap();

    let res = session
        .iclass_query_schedule("64668", &user_id)
        .await
        .unwrap();
    println!("{:#?}", res);

    session.save();
}

#[tokio::test]
async fn test_iclass_checkin() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();
    let user_id = session.iclass_login().await.unwrap();

    let res = session
        .iclass_checkin_schedule("2090542", &user_id)
        .await
        .unwrap();
    println!("{}", res.text().await.unwrap());

    session.save();
}
