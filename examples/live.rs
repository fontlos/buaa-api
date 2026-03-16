#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_live() {
        let context = Context::with_auth("./data").unwrap();

        let live = context.live();

        let schedule = live
            .get_week_schedule("2026-03-09", "2026-03-15")
            .await
            .unwrap();
        println!("Courses: {:?}", schedule);

        context.save_auth("./data").unwrap();
    }
}

fn main() {}
