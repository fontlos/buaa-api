//! Smart Classroom System (iclass) API

use crate::{Session, SessionError, utils, crypto};

impl Session{
    /// iclass Login
    pub async fn iclass_login(&self) -> Result<(), SessionError> {
        // 获取 JSESSIONID
        let res = self.get("https://iclass.buaa.edu.cn:8346/")
            .send()
            .await
            .unwrap();
        let url = res.url().as_str();
        let url = &url[..url.len() - 2];
        // TODO 如果最终 URL 不包含 https://iclass.buaa.edu.cn:8346/, 则重新登录
        if !url.contains("https://iclass.buaa.edu.cn:8346/") {
            return Err(SessionError::LoginError("iclass login failed, maybe sso is expires".to_string()));
        }
        // 使用 DES 加密 URL, 这是下一步请求的参数之一
        let url = crypto::des::des_encrypt(url);
        let params= [
            ("method", "html5GetPrivateUserInfo"),
            ("url", &url),
        ];
        let res = self.get("https://iclass.buaa.edu.cn:8346/wc/auth/html5GetPrivateUserInfo")
            .query(&params)
            .send()
            .await
            .unwrap();
        let html = res.text().await.unwrap();
        println!("{}", html);
        Ok(())
    }

    /// iclass check in
    /// TODO 还缺少一些参数
    pub async fn iclass_checkin(&self, id: &str) -> Result<(), SessionError> {
        let time = utils::get_time();
        let res = self.get(format!("http://iclass.buaa.edu.cn:8081/app/course/stu_scan_sign.action?courseSchedId={}&timestamp={}", id, time))
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

    session.iclass_login().await.unwrap();
    // session.iclass_checkin("2086417").await.unwrap();

    session.save();
}

// http://iclass.buaa.edu.cn:8081/app/course/stu_scan_sign.action?courseSchedId=2086417&timestamp=1731289518963
// http://iclass.buaa.edu.cn:8081/app/course/stu_scan_sign.action?courseSchedId=2086417&timestamp=1731289827114
