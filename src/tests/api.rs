//! # API tests
//! These are passed API. Default ignore

#[cfg(test)]
mod tests {
    use crate::utils::env;
    use crate::Session;

    #[tokio::test]
    async fn test_login() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let mut session = Session::new_in_file("cookie.json");
        session.sso_login(&username, &password).await.unwrap();

        session.save();
    }

    #[tokio::test]
    async fn test_login_uc() {
        let env = crate::utils::env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let mut session = Session::new_in_file("cookie.json");
        session.sso_login(&username, &password).await.unwrap();

        session.uc_login().await.unwrap();
        let state = session.uc_get_state().await.unwrap();
        println!("{}", state);

        session.save();
    }

    #[tokio::test]
    async fn test_gw_login() {
        let env = crate::utils::env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();
        let session = Session::new_in_memory();
        match session.gw_login(&username, &password).await {
            Ok(_) => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
