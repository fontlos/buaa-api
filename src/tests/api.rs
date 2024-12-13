//! # API tests
//! These are passed API. Default ignore

#[cfg(test)]
mod tests {
    use crate::Session;

    #[tokio::test]
    async fn test_user() {
        let env = crate::utils::env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let mut session = Session::new();
        session.with_cookies("cookie.json");

        session.sso_login(&username, &password).await.unwrap();
        session.user_login().await.unwrap();

        let state = session.user_get_state().await.unwrap();
        println!("{}", state);

        session.save();
    }

    #[tokio::test]
    async fn test_wifi_login() {
        let env = crate::utils::env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let session = Session::new();
        session.wifi_login(&username, &password).await.unwrap();
    }

    #[tokio::test]
    async fn test_wifi_logout() {
        let env = crate::utils::env();
        let username = env.get("USERNAME").unwrap();

        let session = Session::new();
        session.wifi_logout(&username).await.unwrap();
    }
}
