#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_spoc() {
        let context = Context::with_auth("./data").unwrap();

        let spoc = context.spoc();

        let res = spoc.get_week().await.unwrap();
        println!("{:?}", res);
        let res = spoc.get_week_schedule(&res).await.unwrap();
        println!("{:?}", res);
        context.save_auth("./data").unwrap();
    }
}

fn main() {}
