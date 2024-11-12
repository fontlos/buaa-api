//! Smart Classroom System (iclass) API

use reqwest::Response;
use serde::Deserialize;

use crate::{Session, SessionError, utils, crypto};

#[derive(Deserialize)]
struct IClassLogin {
    result: IClassLoginResult,
}

#[derive(Deserialize)]
struct IClassLoginResult {
    id: String,
}

impl Session{
    /// Smart Classroom Login
    pub async fn iclass_login(&self) -> Result<String, SessionError> {
        // 获取 JSESSIONID
        let res = self.get("https://iclass.buaa.edu.cn:8346/")
            .send()
            .await?;

        // 整个这一次请求的意义存疑, 但也许是为了验证 loginName 是否有效
        let url = res.url().as_str();
        // TODO 如果获取失败, 说明登录已过期, 则重新登录
        let login_name = match utils::get_value_by_lable(url, "loginName=", "#/") {
            Some(v) => v,
            None => return Err(SessionError::LoginError("iclass login failed, maybe sso is expires".to_string())),
        };
        let url = &url[..url.len() - 2];
        // 使用 DES 加密 URL, 这是下一步请求的参数之一
        let url = crypto::des::des_encrypt(url);
        let params= [
            ("method", "html5GetPrivateUserInfo"),
            ("url", &url),
        ];
        self.get("https://iclass.buaa.edu.cn:8346/wc/auth/html5GetPrivateUserInfo")
            .query(&params)
            .send()
            .await?;

        let params= [
            ("phone", &login_name[..]),
            ("password", ""),
            ("verificationType", "2"),
            ("verificationUrl", ""),
            ("userLevel", "1"),
        ];
        let res = self.get("https://iclass.buaa.edu.cn:8346/app/user/login.action")
            .query(&params)
            .send()
            .await?;
        let res = res.text().await?;
        match serde_json::from_str::<IClassLogin>(&res) {
            Ok(res) => Ok(res.result.id),
            Err(_) => Err(SessionError::LoginError(format!("Smart Classroom Login Failed: {}", res))),
        }
    }

    /// get all courses id
    pub async fn iclass_get_course() -> Result<(), SessionError> {
        Ok(())
    }

    /// get schedule by course id
    pub async fn iclass_get_sche() -> Result<(), SessionError> {
        Ok(())
    }

    /// class check in with schedule id abd user id
    pub async fn iclass_checkin(&self, sche_id: &str, user_id: &str) -> Result<Response, SessionError> {
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
    session.login(&username, &password).await.unwrap();

    let user_id = session.iclass_login().await.unwrap();
    println!("User id: {}", user_id);

    session.save();
}

#[tokio::test]
async fn test_iclass_checkin() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.login(&username, &password).await.unwrap();
    let user_id = session.iclass_login().await.unwrap();

    let res = session.iclass_checkin("2087319", &user_id).await.unwrap();
    println!("{:?}", res.text().await.unwrap());

    session.save();
}
