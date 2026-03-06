#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_srs() {
        let context = Context::with_auth("./data").unwrap();

        let srs = context.srs();
        srs.login().await.unwrap();

        let mut filter = srs.get_default_filter().await.unwrap();
        filter.set_display_conflict(true);
        let res = srs.query_course(&filter).await.unwrap();
        println!("{:#?}", res);

        // let batch = srs.get_batch().await.unwrap();
        // let mut opt = res[0].as_opt();
        // opt.set_batch(&batch);
        // opt.set_index(1);
        // srs.pre_select_course(&opt).await.unwrap();

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
