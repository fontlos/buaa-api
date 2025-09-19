#[cfg(test)]
mod tests {
    use crate::Context;

    #[ignore]
    #[tokio::test]
    async fn test_spoc() {
        let context = Context::with_auth("./data");

        let spoc = context.spoc();

        let res = spoc.get_week().await.unwrap();
        println!("{:?}", res);
        let res = spoc.get_week_schedule(&res).await.unwrap();
        println!("{:?}", res);
        // let res = spoc.get_day_schedule("2025-9-17").await.unwrap();
        // println!("{:?}", res);
        context.save_auth("./data");
    }
}

fn main() {}