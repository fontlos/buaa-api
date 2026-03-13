#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_class_checkin() {
        use buaa_api::time::DateTime;
        let context = Context::with_auth("./data").unwrap();

        let class = context.class();

        let date = DateTime::now();
        let res = class.query_schedule(&date).await.unwrap();
        println!("{:#?}", res);
        class.checkin(&res[0].id).await.unwrap();

        context.save_auth("./data").unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_class_course() {
        let context = Context::with_auth("./data").unwrap();

        let class = context.class();

        let res = class.query_course("202520261").await.unwrap();
        println!("{:#?}", res);
        let res = class.query_course_schedule(&res[0].id).await.unwrap();
        println!("{:#?}", res);
        class.checkin(&res.last().unwrap().id).await.unwrap();

        context.save_auth("./data").unwrap();
    }
}

fn main() {}
