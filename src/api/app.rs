//! BUAA App API
//! APIs for various apps are mixed in here, including class schedules, etc

use crate::{Session, SessionError};

impl Session {
    pub async fn app_login(&self) -> Result<(), SessionError> {
        self.get("https://app.buaa.edu.cn/uc/wap/login")
            .send()
            .await?;
        Ok(())
    }

    pub async fn app_classtable_get_index(&self) -> Result<String, SessionError> {
        let res = self
            .get("https://app.buaa.edu.cn/timetable/wap/default/get-index")
            .send()
            .await?;
        let text = res.text().await?;
        Ok(text)
    }

    pub async fn app_classtable_get_data(&self) -> Result<String, SessionError> {
        let form = [
            ("year", "2024-2025"),
            ("term", "1"),
            ("week", "13"),
            ("type", "2"),
        ];
        let res = self
            .post("https://app.buaa.edu.cn/timetable/wap/default/get-datatmp")
            .form(&form)
            .send()
            .await?;
        let text = res.text().await?;
        Ok(text)
    }
}

#[tokio::test]
async fn test_get_classtable() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();
    session.app_login().await.unwrap();

    let res = session.app_classtable_get_data().await.unwrap();
    println!("{}", res);

    session.save();
}
