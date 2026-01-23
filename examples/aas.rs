#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_aas() {
        let context = Context::with_auth("./data").unwrap();

        let aas = context.aas();

        let config = aas.get_config().await.unwrap();

        let week_schedule = aas.query_week_schedule(&config).await.unwrap();
        println!("{:#?}", week_schedule);
        let term_schedule = aas.query_term_schedule(&config).await.unwrap();
        println!("{:#?}", term_schedule);

        context.save_auth("./data").unwrap();
    }
}

fn main() {}
