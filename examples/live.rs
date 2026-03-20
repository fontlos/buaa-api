#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_live() {
        use buaa_api::time::Week;
        let context = Context::with_auth("./data").unwrap();

        let live = context.live();

        let week = Week::current();

        let schedule = live.get_week_schedule(&week).await.unwrap();
        println!("Courses: {:?}", schedule);

        context.save_auth("./data").unwrap();
    }
}

fn main() {}
