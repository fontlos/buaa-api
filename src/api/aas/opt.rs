use reqwest::Method;

use super::{Config, Data, Res, Schedules};

impl super::AasApi {
    /// # Get user config
    pub async fn get_config(&self) -> crate::Result<Config> {
        let url = "https://byxt.buaa.edu.cn/jwapp/sys/homeapp/api/home/currentUser.do";
        let bytes = self.universal_request(url, Method::GET, &()).await?;
        let config: Data<Config> = Res::parse(&bytes, "Failed to get config")?;
        Ok(config.0)
    }

    /// # Query week schedule
    pub async fn query_week_schedule(&self, config: &Config) -> crate::Result<Schedules> {
        let url =
            "https://byxt.buaa.edu.cn/jwapp/sys/homeapp/api/home/student/getMyScheduleDetail.do";
        let query = [
            ("termCode", config.term.as_str()),
            ("campusCode", ""),
            ("type", "week"),
            ("week", &config.week.to_string()),
        ];
        let bytes = self.universal_request(url, Method::POST, &query).await?;
        let res: Schedules = Res::parse(&bytes, "Failed to get week schedule")?;
        Ok(res)
    }

    /// # Query term schedule
    pub async fn query_term_schedule(&self, config: &Config) -> crate::Result<Schedules> {
        let url =
            "https://byxt.buaa.edu.cn/jwapp/sys/homeapp/api/home/student/getMyScheduleDetail.do";
        let query = [
            ("termCode", config.term.as_str()),
            ("campusCode", ""),
            ("type", "term"),
        ];
        let bytes = self.universal_request(url, Method::POST, &query).await?;
        let res: Schedules = Res::parse(&bytes, "Failed to get term schedule")?;
        Ok(res)
    }
}
