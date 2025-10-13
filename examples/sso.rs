#[tokio::test]
async fn test_sso_login() {
    use buaa_api::Context;
    use logforth::append;
    use logforth::record::LevelFilter;

    logforth::starter_log::builder()
        .dispatch(|d| {
            d.filter(LevelFilter::Info)
                .append(append::Stdout::default())
        })
        .apply();

    let context = Context::with_auth("./data");

    let sso = context.sso();

    sso.login().await.unwrap();

    context.save_auth("./data");
}

fn main() {}
