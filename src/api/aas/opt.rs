use reqwest::Method;

use super::{Res, Data, Config};

impl super::AasApi {
    /// # Get user config
    pub async fn get_config(&self) -> crate::Result<Config> {
        let url = "https://byxt.buaa.edu.cn/jwapp/sys/homeapp/api/home/currentUser.do";
        let bytes = self.universal_request(url, Method::GET, &()).await?;
        let config: Data<Config> = Res::parse(&bytes, "Failed to get config")?;
        Ok(config.0)
    }
}
