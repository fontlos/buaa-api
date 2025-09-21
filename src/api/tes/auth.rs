use crate::api::Sso;
use crate::error::Error;

impl super::TesApi {
    /// Teacher Evaluation System Login
    pub async fn login(&self) -> crate::Result<()> {
        let cred = self.cred.load();
        if cred.is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        // 登录
        let login_url =
            "https://sso.buaa.edu.cn/login?service=https%3A%2F%2Fspoc.buaa.edu.cn%2Fpjxt%2Fcas";
        let res = self.client.get(login_url).send().await?;
        if res.url().as_str() == login_url {
            return Err(Error::server("[Tes] Redirect failed"));
        }
        cred.refresh::<Sso>();
        Ok(())
    }
}
