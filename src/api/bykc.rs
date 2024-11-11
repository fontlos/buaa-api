//! Boya Course API

use crate::{Session, SessionError};

impl Session{
    /// bykc Login</br>
    /// return auth token
    pub async fn bykc_login(&self) -> Result<String, SessionError> {
        // 获取 JSESSIONID
        let res = self
            .get("https://sso.buaa.edu.cn/login?noAutoRedirect=true&service=https%3A%2F%2Fbykc.buaa.edu.cn%2Fsscv%2Fcas%2Flogin")
            .send()
            .await
            .unwrap();
        let url = res.url().as_str();
        let start = url.find("token=").unwrap() + "token=".len();
        let token = &url[start..];
        Ok(token.to_string())
    }
    // ak, sk, ts 这些是请求头里的重要信息
    // 请求负载 iycXIEsM7EVqBeoaDRFv/5ehK1MAM5BPWtq/iasLJ9E=, 疑似aes加密
    // auth_token authtoken 为 bykc_login 获取的 token
    pub async fn bykc_query_all_course(&self) -> Result<(), SessionError> {
        // 获取 JSESSIONID
        let res = self.post("https://bykc.buaa.edu.cn/sscv/queryStudentSemesterCourseByPage")
            .send()
            .await
            .unwrap();
        let url = res.url().as_str();
        println!("{}", url);
        println!("{}", res.text().await.unwrap());
        Ok(())
    }
}

#[tokio::test]
async fn test_bykc_login() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.login(&username, &password).await.unwrap();

    let token = session.bykc_login().await.unwrap();
    println!("{}", token);

    session.save();
}
