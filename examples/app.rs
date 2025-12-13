#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_app() {
        let context = Context::with_auth("./data").unwrap();

        let app = context.app();

        let exams = app.get_exam().await.unwrap();
        println!("{:#?}", exams);

        context.save_auth("./data").unwrap();
    }
}

fn main() {}
