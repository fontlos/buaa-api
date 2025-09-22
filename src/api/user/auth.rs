use crate::api::Sso;
use crate::utils;

impl super::UserApi {
    /// # Login to UserApi
    pub async fn login(&self) -> crate::Result<()> {
        let cred = self.cred.load();
        if cred.is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        let time = utils::get_time_millis();
        // 获取 JSESSIONID
        self.client
            .get(format!(
                "https://uc.buaa.edu.cn/api/uc/status?selfTimestamp={time}"
            ))
            .send()
            .await?;
        // 验证  JSESSIONID
        // 会经历 4 次重定向
        self.client
            .get("https://uc.buaa.edu.cn/api/login?target=https://uc.buaa.edu.cn/#/user/login")
            .send()
            .await?;
        cred.refresh::<Sso>();
        Ok(())
    }
}
