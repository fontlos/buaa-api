#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_srs() {
        let context = Context::with_auth("./data").unwrap();

        let srs = context.srs();

        let filter = srs.get_default_filter().await.unwrap();
        let res = srs.query_course(&filter).await.unwrap();
        let batch = srs.get_batch().await.unwrap();
        println!("{:?}", res);
        let mut opt = res.data[0].as_opt();
        opt.set_batch(&batch);
        opt.set_index(1);
        srs.pre_select_course(&opt).await.unwrap();

        context.save_auth("./data").unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_selected() {
        let context = Context::with_auth("./data").unwrap();

        let srs = context.srs();

        let res = srs.query_selected().await.unwrap();
        println!("{:?}", res);

        context.save_auth("./data").unwrap();
    }
}

fn main() {}
