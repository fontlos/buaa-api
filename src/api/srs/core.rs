use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::api::{Srs, Sso};
use crate::error::Error;

use super::Body;

impl super::SrsApi {
    /// Login to SrsApi
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        let url = "https://sso.buaa.edu.cn/login?service=https%3A%2F%2Fbyxk.buaa.edu.cn%2Fxsxk%2Fauth%2Fcas";
        // 获取 JSESSIONID
        let res = self.client.get(url).send().await?;
        // 未转跳就证明登录过期
        if res.url().as_str() == url {
            return Err(Error::server("Redirect failed").with_label("Srs"));
        }
        // 储存 token
        let cookie = self.cookies.load();
        match cookie.get("byxk.buaa.edu.cn", "/xsxk", "token") {
            Some(t) => {
                self.cred.update(|s| {
                    s.update::<Srs>(t.value().to_string());
                });
                Ok(())
            }
            None => Err(Error::server("Login failed. No Token").with_label("Srs")),
        }
    }

    pub(super) async fn universal_request<'a, Q, T>(
        &self,
        url: &str,
        body: Body<'a, Q>,
    ) -> crate::Result<T>
    where
        Q: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let cred = &self.cred.load();
        if cred.is_expired::<Srs>() {
            self.login().await?;
        }
        let token = cred.value::<Srs>()?;

        let res = self.client.post(url).header("Authorization", token);

        let res = match body {
            Body::QueryToken => res.query(&[("token", token)]),
            Body::Form(f) => res.form(f),
            Body::Json(j) => res.json(j),
            Body::None => res,
        };

        let res = res.send().await?.bytes().await?;
        let res = serde_json::from_slice::<Res<T>>(&res)?;
        if res.code != 200 {
            return Err(Error::server(format!("Response: {}", res.msg)).with_label("Srs"));
        }
        Ok(res.data)
    }
}

#[derive(Deserialize)]
struct Res<T> {
    code: u16,
    msg: String,
    data: T,
}
