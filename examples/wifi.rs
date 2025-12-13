#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_login() {
        let context = Context::with_auth("./data").unwrap();

        let wifi = context.wifi();
        wifi.login().await.unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_logout() {
        let context = Context::with_auth("./data").unwrap();

        let wifi = context.wifi();
        wifi.logout().await.unwrap();
    }
}

fn main() {}
