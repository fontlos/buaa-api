use crate::api::{Live, Sso};
use crate::error::Error;

impl super::LiveApi {
    /// # Login to LiveApi
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }
        let url = "https://yjapi.msa.buaa.edu.cn/casapi/index.php?r=auth/login&auType=cmc&tenant_code=21&forward=https%3A%2F%2Fclassroom.msa.buaa.edu.cn";
        let verify_url = "https://classroom.msa.buaa.edu.cn/";
        let res = self.client.get(url).send().await?;
        if res.url().as_str() != verify_url {
            let text = res.text().await?;
            return Err(Error::server("Login failed")
                .with_label("Live")
                .with_source(text));
        }
        match self
            .cookies
            .load()
            .get("msa.buaa.edu.cn", "_token")
            .and_then(|c| c.value())
        {
            Some(t) => {
                // 这里的 t 来自 PHP 数组, 使用 URL 编码
                // 原始格式 a:2:{i:0;s:6:"_token";i:1;s:[TOKEN_LENGTH]:"[TOKEN]";}
                // 这是一个包含两个元素的数组, 字符串 "_token" 与实际的 token 值, i 标记索引, s 标记字符串长度
                // 只需倒序查找两次 "%22" 即 '"'
                let err = || Error::server("Login failed. Invalid token format").with_label("Live");
                let end = t.rfind("%22").ok_or_else(err)?;
                let start = t[..end].rfind("%22").ok_or_else(err)?;
                let token = &t[start + 3..end];
                self.cred.update(|s| {
                    s.update::<Live>(token.to_string());
                });
                Ok(())
            }
            None => Err(Error::server("Login failed. No token").with_label("Live")),
        }
    }
}
