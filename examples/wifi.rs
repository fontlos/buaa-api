#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_wifi_login() {
        let context = Context::with_auth("./data");

        let wifi = context.wifi();
        wifi.login().await.unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_wifi_logout() {
        let context = Context::with_auth("./data");

        let wifi = context.wifi();
        wifi.logout().await.unwrap();
    }
}

fn main() {}