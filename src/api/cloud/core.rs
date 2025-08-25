use serde_json::Value;

use crate::api::Location;
use crate::error::Error;

impl super::CloudApi {
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().sso.is_expired() {
            self.api::<crate::api::Sso>().login().await?;
        }

        let mut is_ok = false;

        // TODO: 这有一个可能的 bug
        // 如果 302 到 `signin` 后可解析出登陆参数 login_challenge
        // 如果 302 到 `callback` 可以使用 refresh_token
        // 问题出在如果刷新 token 过期了会怎样, 会重定向到 `signin` 吗
        // 目前不知道其有效期
        let url =
            "https://bhpan.buaa.edu.cn/anyshare/oauth2/login?redirect=%2Fanyshare%2Fzh-cn%2Fportal";
        let res = self.client.get(url).send().await?;
        let path = res.url().path();

        if path == "/oauth2/signin" {
            // 从 URL 解析 login_challenge=xxx
            let login_challenge = res.url().query().unwrap().to_string();

            let login_challenge = cookie_store::Cookie::parse(
                login_challenge,
                &"https://bhpan.buaa.edu.cn/".parse().unwrap(),
            )
            .unwrap();

            // 这里有一条需要手动添加的临时 Cookie
            self.cookies.update(|store| {
                store
                    .insert(
                        login_challenge,
                        &"https://bhpan.buaa.edu.cn/".parse().unwrap(),
                    )
                    .unwrap();
            });
            // 发起登录请求
            let url =
                "https://sso.buaa.edu.cn/login?service=https://bhpan.buaa.edu.cn/oauth2/signin";
            let res = self.client.get(url).send().await?;
            // 来到回调地址证明登陆成功
            if res.url().path() == "/anyshare/oauth2/login/callback" {
                is_ok = true;
            }
            // 移除临时 Cookie
            self.cookies.update(|store| {
                store.remove("bhpan.buaa.edu.cn", "/", "login_challenge");
            })
        } else if path == "/anyshare/oauth2/login/callback" {
            let url = "https://bhpan.buaa.edu.cn/anyshare/oauth2/login/refreshToken";
            self.client.get(url).send().await?;
            is_ok = true;
        }

        if is_ok {
            // is_previous_login_3rd_party=true 和 oauth2.isSkip=true 两个 cookie 似乎没有用, 这里就不添加了
            let token =
                match self
                    .cookies
                    .load()
                    .get("bhpan.buaa.edu.cn", "/", "client.oauth2_token")
                {
                    Some(t) => t.value().to_string(),
                    None => {
                        return Err(Error::server("[Cloud] Login failed. No token"));
                    }
                };
            self.cred.set(Location::Cloud, token);
            Ok(())
        } else {
            Err(Error::server("[Cloud] Login failed. Unknown error"))
        }
    }

    pub(crate) async fn token(&self) -> crate::Result<&String> {
        let cred = &self.cred.load().cloud_token;
        if cred.is_expired() {
            self.login().await?;
        }
        cred.value()
            .ok_or_else(|| Error::auth_expired(Location::Cloud))
    }

    pub async fn universal_request(&self, url: &str, data: &Value) -> crate::Result<String> {
        let token = self.token().await?;

        let res = self.post(url).bearer_auth(token).json(data).send().await?;
        let text = res.text().await?;

        Ok(text)
    }
}
