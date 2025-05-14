use crate::Error;

impl super::CloudAPI {
    pub async fn login(&self) -> crate::Result<()> {
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
            store.insert(
                login_challenge,
                &"https://bhpan.buaa.edu.cn/".parse().unwrap(),
            ).unwrap();
        });

        // 发起登录请求
        let url = "https://sso.buaa.edu.cn/login?service=https://bhpan.buaa.edu.cn/oauth2/signin";
        let res = self.client.get(url).send().await?;

        // 来到回调地址证明登陆成功
        if res.url().path() == "/anyshare/oauth2/login/callback" {
            // is_previous_login_3rd_party=true 和 oauth2.isSkip=true 两个 cookie 似乎没有用, 这里就不添加了
            // 至于用于操作的 client.oauth2_token 既然已经在 cookie 中了, 就不单独复制一份到 config 里了
            Ok(())
        } else {
            Err(Error::LoginError("Login failed".to_string()))
        }
    }
}
