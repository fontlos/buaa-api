//! Smart Classroom System (iclass) API

use reqwest::Response;
use crate::{Session, SessionError, utils, crypto};

impl Session{
    /// iclass Login
    pub async fn iclass_login(&self) -> Result<Response, SessionError> {
        // 获取 JSESSIONID
        let res = self.get("https://iclass.buaa.edu.cn:8346/")
            .send()
            .await
            .unwrap();

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
            .await
            .unwrap();

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
            .await
            .unwrap();
        Ok(res)
    }

    // TODO 还缺少一些参数
    /// iclass check in
    pub async fn iclass_checkin(&self, id: &str) -> Result<(), SessionError> {
        // http://iclass.buaa.edu.cn:8081/app/course/stu_scan_sign.action?courseSchedId=2086417&timestamp=1731289518963
        // http://iclass.buaa.edu.cn:8081/app/course/stu_scan_sign.action?courseSchedId=2086417&timestamp=1731289827114

        let time = utils::get_time();
        let res = self.post(format!("http://iclass.buaa.edu.cn:8081/app/course/stu_scan_sign.action?courseSchedId={}&timestamp={}", id, time))
            .send()
            .await
            .unwrap();
        println!("{}", res.url().as_str());
        println!("{}", res.text().await.unwrap());
        Ok(())
    }
}

#[tokio::test]
async fn test_iclass_login() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.login(&username, &password).await.unwrap();

    let res = session.iclass_login().await.unwrap();
    let text = res.text().await.unwrap();
    println!("{}", text);

    session.save();
}

#[tokio::test]
async fn test_iclass_checkin() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.login(&username, &password).await.unwrap();
    session.iclass_login().await.unwrap();

    session.iclass_checkin("2086417").await.unwrap();

    session.save();
}
