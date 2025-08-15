use crate::api::Location;

impl super::AppApi {
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().sso.is_expired() {
            self.api::<crate::api::Sso>().login().await?;
        }

        self.get("https://app.buaa.edu.cn/uc/wap/login")
            .send()
            .await?;

        self.cred.refresh(Location::Sso);

        Ok(())
    }
}
