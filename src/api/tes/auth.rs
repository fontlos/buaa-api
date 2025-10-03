use crate::api::Sso;
use crate::error::Error;

impl super::TesApi {
    /// Login to TesApi
    pub async fn login(&self) -> crate::Result<()> {
        let cred = self.cred.load();
        if cred.is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        let url =
            "https://sso.buaa.edu.cn/login?service=https%3A%2F%2Fspoc.buaa.edu.cn%2Fpjxt%2Fcas";
        let res = self.client.get(url).send().await?;
        if res.url().as_str() == url {
            return Err(Error::server("Redirect failed").with_label("Tes"));
        }
        cred.refresh::<Sso>();
        Ok(())
    }
}
