#[cfg(test)]
mod tests {
    use buaa_api::Context;
    use buaa_api::utils;

    #[ignore]
    #[tokio::test]
    async fn test_boya() {
        let context = Context::with_auth("./data");
        // 2025.5.18 15:00 我们也成功支持 SSO 的自动刷新了
        // 现在真正可以直接调用 API 无需预处理了
        // context.login().await.unwrap();

        let boya = context.boya();
        // 2025.5.17 14:00 现在至少 Boya 的 API 是支持自动刷新的
        // boya.login().await.unwrap();

        let res = boya.query_course().await.unwrap();
        println!("{:?}", res);

        context.save_auth("./data");
    }

    #[ignore]
    #[tokio::test]
    async fn test_sign_rule() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let res = boya.query_sign_rule(7882).await.unwrap();
        println!("{:?}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_selected() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let start = utils::parse_date("2024-08-26");
        let end = utils::parse_date("2024-12-29");

        let res = boya.query_selected(start, end).await.unwrap();
        println!("{:?}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_select() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let res = boya.select_course(6637).await.unwrap();
        println!("{}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_drop() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let res = boya.drop_course(6637).await.unwrap();
        println!("{}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_statistic() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let res = boya.query_statistic().await.unwrap();
        println!("{}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_checkin_checkout() {
        let context = Context::with_auth("./data");

        let boya = context.boya();
        let id = 7774;

        let rule = boya.query_sign_rule(id).await.unwrap().unwrap();
        println!("{:?}", rule);

        let time = utils::get_datatime();
        if rule.checkin_start < time && time < rule.checkin_end {
            let res = boya.checkin_course(id, &rule.coordinate).await.unwrap();
            println!("Checkin: {:?}", res);
            return;
        }

        if rule.checkout_start < time && time < rule.checkout_end {
            let res = boya.checkout_course(id, &rule.coordinate).await.unwrap();
            println!("Checkout: {:?}", res);
            return;
        }
    }
}

fn main() {}
