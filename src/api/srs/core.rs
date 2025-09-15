use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::api::Location;
use crate::api::{Srs, Sso};
use crate::error::Error;

use super::{_SrsBody, _SrsRes};

impl super::SrsApi {
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        let url = "https://sso.buaa.edu.cn/login?service=https%3A%2F%2Fbyxk.buaa.edu.cn%2Fxsxk%2Fauth%2Fcas";
        // 获取 JSESSIONID
        let res = self.get(url).send().await?;
        // 未转跳就证明登录过期
        if res.url().as_str() == url {
            return Err(Error::auth_expired(Location::Sso));
        }
        // 储存 token
        let cookie = self.cookies.load();
        match cookie.get("byxk.buaa.edu.cn", "/xsxk", "token") {
            Some(t) => {
                self.cred.set(Location::Srs, t.value().to_string());
                Ok(())
            }
            None => Err(Error::server("[Srs] Login failed. No Token")),
        }
    }

    pub(super) async fn universal_request<'a, Q, T>(
        &self,
        url: &str,
        body: _SrsBody<'a, Q>,
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

        let res = self.post(url).header("Authorization", token);

        let res = match body {
            _SrsBody::QueryToken => res.query(&[("token", token)]),
            _SrsBody::Form(f) => res.form(f),
            _SrsBody::Json(j) => res.json(j),
            _SrsBody::None => res,
        };

        let res = res.send().await?.bytes().await?;
        let res = serde_json::from_slice::<_SrsRes<T>>(&res)?;
        if res.code != 200 {
            return Err(Error::server(format!("[Srs] Response: {}", res.msg)));
        }
        Ok(res.data)
    }
}
