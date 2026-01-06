use crate::api::Sso;
use crate::error::Error;

impl super::AasApi {
    /// # Login to AasApi
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        let login_url = "https://sso.buaa.edu.cn/login?service=https%3A%2F%2Fbyxt.buaa.edu.cn%2Fjwapp%2Fsys%2Fhomeapp%2Findex.do%3FcontextPath%3D%2Fjwapp";
        let verify_url = "https://byxt.buaa.edu.cn/jwapp/sys/homeapp/index.do?contextPath=/jwapp";
        let res = self.client.get(login_url).send().await?;
        if res.url().as_str() != verify_url {
            return Err(Error::server("Login failed").with_label("Aas"));
        }
        Ok(())
    }
}
