use bytes::Bytes;
use reqwest::Method;

use crate::api::{Aas, Sso};
use crate::error::Error;

impl super::AasApi {
    /// # Login to AasApi
    pub async fn login(&self) -> crate::Result<()> {
        let cred = self.cred.load();
        if cred.is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        let login_url = "https://sso.buaa.edu.cn/login?service=https%3A%2F%2Fbyxt.buaa.edu.cn%2Fjwapp%2Fsys%2Fhomeapp%2Findex.do%3FcontextPath%3D%2Fjwapp";
        // 或者有 &ticket=xxx
        let verify_url = "https://byxt.buaa.edu.cn/jwapp/sys/homeapp/index.do?contextPath=/jwapp";
        let res = self.client.get(login_url).send().await?;
        if !res.url().as_str().starts_with(verify_url) {
            return Err(Error::server("Login failed").with_label("Aas"));
        }
        cred.refresh::<Sso>();
        cred.refresh::<Aas>();
        Ok(())
    }

    /// # Universal request for AasApi
    pub async fn universal_request<P>(
        &self,
        url: &str,
        method: Method,
        payload: &P,
    ) -> crate::Result<Bytes>
    where
        P: serde::Serialize + ?Sized,
    {
        let cred = self.cred.load();
        if cred.is_expired::<Aas>() {
            self.login().await?;
        }

        let res = self
            .client
            .request(method, url)
            .query(&payload)
            .send()
            .await?;
        Ok(res.bytes().await?)
    }
}
