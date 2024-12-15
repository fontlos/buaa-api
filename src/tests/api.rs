//! # API tests
//! These are passed API. Default ignore

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_user() {
        let env = crate::utils::env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = crate::Context::new();
        context.set_account(username, password);
        context.with_cookies("cookie.json");
        context.login().await.unwrap();

        let user = context.user();
        user.login().await.unwrap();

        let state = user.get_state().await.unwrap();
        println!("{}", state);

        context.save();
    }

    #[tokio::test]
    async fn test_wifi_login() {
        let env = crate::utils::env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = crate::Context::new();
        context.set_account(username, password);

        let wifi = context.wifi();
        wifi.login().await.unwrap();
    }

    #[tokio::test]
    async fn test_wifi_logout() {
        let env = crate::utils::env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = crate::Context::new();
        context.set_account(username, password);

        let wifi = context.wifi();
        wifi.logout().await.unwrap();
    }
}
