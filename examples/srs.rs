#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_srs() {
        let context = Context::with_auth("./data");

        let srs = context.srs();

        let filter = srs.get_default_filter().await.unwrap();

        let res = srs.query_course(&filter).await.unwrap();
        println!("{:?}", res);

        let batch = srs.get_batch().await.unwrap();

        srs.pre_select_course(&res.data[0], &filter, &batch, 1)
            .await
            .unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_selected() {
        let context = Context::with_auth("./data");

        let srs = context.srs();

        let res = srs.query_selected().await.unwrap();

        println!("{:?}", res);
    }
}

fn main() {}
