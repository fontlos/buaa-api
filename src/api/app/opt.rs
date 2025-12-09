use crate::{Error, utils};

use super::Exams;

impl super::AppApi {
    /// Get exam schedule
    pub async fn get_exam(&self) -> crate::Result<Exams> {
        let url = "https://app.buaa.edu.cn/exam/wap/default/index";
        let bytes = self.universal_request(url).await?;

        // JSON 硬嵌在 HTML 里是真没招了. 可能是个不稳定的解析, 依赖于服务端渲染的稳定度
        match utils::parse_by_tag(&bytes, "\n        data: ", ",\n")
            .map(serde_json::from_str::<Exams>)
        {
            Some(e) => Ok(e?),
            None => Err(Error::server("Failed to parse exam data").with_label("App")),
        }
    }
}
