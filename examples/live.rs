#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_live() {
        let context = Context::with_auth("./data").unwrap();

        let live = context.live();

        let user = live.get_user_id().await.unwrap();
        println!("User ID: {}", user);

        context.save_auth("./data").unwrap();
    }
}

fn main() {}
