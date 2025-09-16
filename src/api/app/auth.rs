use crate::api::Sso;

impl super::AppApi {
    pub async fn login(&self) -> crate::Result<()> {
        let cred = self.cred.load();
        if cred.is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        self.get("https://app.buaa.edu.cn/uc/wap/login")
            .send()
            .await?;

        cred.refresh::<Sso>();

        Ok(())
    }
}
