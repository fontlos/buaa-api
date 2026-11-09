#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_class_checkin() {
        use buaa_api::ContextBuilder;
        use buaa_api::api::sso::captcha::{CaptchaError, CaptchaSolver};
        use buaa_api::store::cookies::CookieStore;
        use buaa_api::store::cred::CredentialStore;
        use buaa_api::time::DateTime;

        use std::path::Path;
        use std::sync::Arc;

        struct StdinSolver;

        impl CaptchaSolver for StdinSolver {
            fn solve(&self, captcha: &[u8]) -> Result<String, CaptchaError> {
                std::fs::write("./data/captcha.jpg", &captcha).unwrap();
                println!("Captcha saved to ./data/captcha.jpg");
                println!("Please input captcha: ");
                let mut s = String::new();
                std::io::stdin()
                    .read_line(&mut s)
                    .map_err(|_| CaptchaError::SolveFailed)?;
                Ok(s.trim().to_string())
            }
        }

        let dir = Path::new("./data");
        let cookies_path = dir.join("cookies.json");
        let cookies = CookieStore::from_file(cookies_path).unwrap();
        let cred_path = dir.join("cred.json");
        let cred = CredentialStore::from_file(cred_path).unwrap();
        let context = ContextBuilder::new()
            .cookies(cookies)
            .cred(cred)
            .captcha(Arc::new(StdinSolver))
            .build();

        // vpn下大于1.5小时小于2小时
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

        let res = class.query_course("2026202701").await.unwrap();
        println!("{:#?}", res);
        let res = class.query_course_schedule(&res[0].id).await.unwrap();
        println!("{:#?}", res);
        class.checkin(&res.last().unwrap().id).await.unwrap();

        context.save_auth("./data").unwrap();
    }
}

fn main() {}
