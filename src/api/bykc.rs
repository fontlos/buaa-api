//! Boya Course API

use crate::{Session, SessionError};

impl Session{
    /// bykc Login
    pub async fn bykc_login(&self) -> Result<(), SessionError> {
        // 获取 JSESSIONID
        let res = self.get("https://sso.buaa.edu.cn/login?noAutoRedirect=true&service=https%3A%2F%2Fbykc.buaa.edu.cn%2Fsscv%2Fcas%2Flogin")
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

    session.bykc_login().await.unwrap();

    session.save();
}
