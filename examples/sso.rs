#[tokio::test]
async fn test_sso_login() {
    use buaa_api::Context;

    logforth::builder()
        .dispatch(|d| d.append(logforth::append::Stderr::default()))
        .apply();

    let context = Context::with_auth("./data");

    let sso = context.sso();

    sso.login().await.unwrap();

    context.save_auth("./data");
}

fn main() {}
