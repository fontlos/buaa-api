#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_boya() {
        let context = Context::with_auth("./data").unwrap();
        // 2025.5.18 15:00 我们也成功支持 SSO 的自动刷新了
        // 现在真正可以直接调用 API 无需预处理了
        // context.login().await.unwrap();

        let boya = context.boya();
        // 2025.5.17 14:00 现在至少 Boya 的 API 是支持自动刷新的
        // boya.login().await.unwrap();

        let res = boya.query_course().await.unwrap();
        println!("{:?}", res);

        context.save_auth("./data").unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_sign_rule() {
        let context = Context::with_auth("./data").unwrap();

        let boya = context.boya();

        let res = boya.query_sign_rule(8861).await.unwrap();
        println!("{:?}", res);

        context.save_auth("./data").unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_selected() {
        use time::Date;
        use time::macros::format_description;

        let context = Context::with_auth("./data").unwrap();

        let boya = context.boya();

        let format_string = format_description!("[year]-[month]-[day]");

        let start = Date::parse("2024-08-26", &format_string).unwrap();
        let end = Date::parse("2024-12-29", &format_string).unwrap();

        let res = boya.query_selected(Some((start, end))).await.unwrap();
        println!("{:?}", res);

        context.save_auth("./data").unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_select() {
        let context = Context::with_auth("./data").unwrap();

        let boya = context.boya();

        boya.select_course(6637).await.unwrap();

        context.save_auth("./data").unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_drop() {
        let context = Context::with_auth("./data").unwrap();

        let boya = context.boya();

        boya.drop_course(6637).await.unwrap();

        context.save_auth("./data").unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_statistic() {
        let context = Context::with_auth("./data").unwrap();

        let boya = context.boya();

        let res = boya.query_statistic().await.unwrap();
        println!("{:?}", res);

        context.save_auth("./data").unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_checkin_checkout() {
        use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

        let context = Context::with_auth("./data").unwrap();

        let boya = context.boya();
        let id = 7774;

        let rule = boya.query_sign_rule(id).await.unwrap().unwrap();
        println!("{:?}", rule);

        let now_utc = OffsetDateTime::now_utc();
        let local_offset = UtcOffset::from_hms(8, 0, 0).expect("Offset should always be valid");
        let now_local = now_utc.to_offset(local_offset);
        let time = PrimitiveDateTime::new(now_local.date(), now_local.time());

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

        context.save_auth("./data").unwrap();
    }
}

fn main() {}
