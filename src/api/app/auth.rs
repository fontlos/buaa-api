use crate::api::Location;
use crate::api::Sso;

impl super::AppApi {
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        self.get("https://app.buaa.edu.cn/uc/wap/login")
            .send()
            .await?;

        self.cred.refresh(Location::Sso);

        Ok(())
    }
}
