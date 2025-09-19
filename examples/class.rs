#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_class() {
        let context = Context::with_auth("./data");

        let class = context.class();

        let res = class.query_course("202520261").await.unwrap();
        println!("{:#?}", res);
        let res = class.query_schedule(res[0].id).await.unwrap();
        println!("{:#?}", res);
        let res = class.checkin(res.last().unwrap().id).await.unwrap();
        println!("{}", res);

        context.save_auth("./data");
    }
}

fn main() {}
