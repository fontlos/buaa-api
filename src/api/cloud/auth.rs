use crate::error::{Error, Location};

impl super::CloudApi {
    pub async fn login(&self) -> crate::Result<()> {
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && self.cred.load().sso.is_expired() {
            self.api::<crate::api::Core>().login().await?;
        }

        // 获取登录参数, 302 后可解析出 login_challenge
        let url =
            "https://bhpan.buaa.edu.cn/anyshare/oauth2/login?redirect=%2Fanyshare%2Fzh-cn%2Fportal";
        let res = self.client.get(url).send().await?;

        // 如果直接跳转到登录回调地址, 说明已经登录状态仍在
        if res.url().path() == "/anyshare/oauth2/login/callback" {
            return Ok(());
        }

        // 从 URL 解析 login_challenge=xxx
        let login_challenge = res.url().query().unwrap().to_string();

        let login_challenge = cookie_store::Cookie::parse(
            login_challenge,
            &"https://bhpan.buaa.edu.cn/".parse().unwrap(),
        )
        .unwrap();

        // 这里有一条需要手动添加的 cookie
        self.cookies.update(|store| {
            store
                .insert(
                    login_challenge,
                    &"https://bhpan.buaa.edu.cn/".parse().unwrap(),
                )
                .unwrap();
        });

        // 发起登录请求
        let url = "https://sso.buaa.edu.cn/login?service=https://bhpan.buaa.edu.cn/oauth2/signin";
        let res = self.client.get(url).send().await?;

        // 来到回调地址证明登陆成功
        if res.url().path() == "/anyshare/oauth2/login/callback" {
            // is_previous_login_3rd_party=true 和 oauth2.isSkip=true 两个 cookie 似乎没有用, 这里就不添加了
            let token =
                match self
                    .cookies
                    .load()
                    .get("bhpan.buaa.edu.cn", "/", "client.oauth2_token")
                {
                    Some(t) => t.value().to_string(),
                    None => return Err(Error::Server("No Cloud Token".to_string())),
                };
            self.cred.update(|c| {
                // TODO: 我们先默认十分钟过期, 待测试
                c.cloud_token.set(token, 600);
                // 刷新 SSO 时效
                c.sso.refresh(5400);
            });
            Ok(())
        } else {
            Err(Error::Server("Login failed".to_string()))
        }
    }

    pub(crate) async fn token(&self) -> crate::Result<&String> {
        let cred = &self.cred.load().cloud_token;
        // 因为我们可以知道 Token 是否过期, 我们这里只完成保守的刷新, 仅在 Token 超出我们预期时刷新 Token
        if self.policy.load().is_auto() && cred.is_expired() {
            self.login().await?;
        }
        cred.value()
            .ok_or_else(|| Error::auth_expired(Location::Cloud))
    }
}
