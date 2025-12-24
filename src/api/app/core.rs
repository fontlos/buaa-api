use bytes::Bytes;
use reqwest::header::USER_AGENT;

use crate::api::{App, Sso};
use crate::error::Error;

const APP_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 MicroMessenger/7.0.20.1781(0x6700143B) NetType/WIFI MiniProgramEnv/Windows WindowsWechat/WMPF WindowsWechat(0x63090a13) UnifiedPCWindowsWechat(0xf2541022) XWEB/16467";

// API 列表
// https://app.buaa.edu.cn/appsquare/api/app/index?number=&sid=16

impl super::AppApi {
    /// # Login to AppApi
    pub async fn login(&self) -> crate::Result<()> {
        let cred = self.cred.load();
        if cred.is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }
        let login_url = "https://app.buaa.edu.cn/uc/wap/minigram/cas-login?login_from=xiaochengxu";
        let verify_url = "https://app.buaa.edu.cn/uc/wap/minigram/cas-login?redirect=https%3A%2F%2Fapp.buaa.edu.cn%2Fsite%2Fcenter%2Fpersonal&login_from=xiaochengxu";
        let res = self
            .client
            .get(login_url)
            .header(USER_AGENT, APP_UA)
            .send()
            .await?;
        if res.url().as_str() != verify_url {
            let text = res.text().await?;
            return Err(Error::server("Login failed")
                .with_label("App")
                .with_source(text));
        }
        cred.refresh::<Sso>();
        cred.refresh::<App>();
        Ok(())
    }

    /// # Universal Request for AppApi
    pub async fn universal_request(&self, url: &str) -> crate::Result<Bytes> {
        let cred = self.cred.load();
        if cred.is_expired::<App>() {
            self.login().await?;
        }

        let res = self
            .client
            .get(url)
            .header(USER_AGENT, APP_UA)
            .send()
            .await?;

        Ok(res.bytes().await?)
    }
}
