use reqwest::Method;

use super::{Course, Data, Homework, HomeworkDetail, Payload, Res, Schedule, Week};

impl super::SpocApi {
    /// Get current week
    pub async fn get_week(&self) -> crate::Result<Week> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryOne";
        // SQL ID 是固定值, 应该是对应的数据库键什么的
        let json = serde_json::json!({
            "sqlid": "17275975753144ed8d6fe15425677f752c936d97de1bab76"
        });
        let payload = Payload::Json(&json);
        let bytes = self.universal_request(url, Method::POST, payload).await?;
        let res: Week = Res::parse(&bytes)?;
        Ok(res)
    }

    /// Query schedule of a week
    pub async fn query_week_schedules(&self, week: &Week) -> crate::Result<Vec<Schedule>> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/jxkj/queryRlData";
        let query = [
            ("rllx", "1"), // 日历类型
            ("zksrq", &week.date.0),
            ("zjsrq", &week.date.1),
        ];
        let payload = Payload::Query(&query);
        let bytes = self.universal_request(url, Method::GET, payload).await?;
        let res: Vec<Schedule> = Res::parse(&bytes)?;
        Ok(res)
    }

    /// # Query courses
    ///
    /// `term` format: "yyyy-yyyyt", e.g. "2025-20261" for 2025 fall semester.
    /// Can get from [Week::term]
    pub async fn query_courses(&self, term: &str) -> crate::Result<Vec<Course>> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/jxkj/queryKclb";
        let query = [("xnxq", term)];
        let payload = Payload::Query(&query);
        let bytes = self.universal_request(url, Method::GET, payload).await?;
        let res: Vec<Course> = Res::parse(&bytes)?;
        Ok(res)
    }

    /// Query homeworks
    pub async fn query_homeworks(&self, course: &Course) -> crate::Result<Vec<Homework>> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/kczy/queryXsZyList";
        // 有缓存的情况下没有前两个参数也正常, 但没缓存就会返回 Null
        let query = [("flag", "1"), ("sflx", "2"), ("sskcid", &course.id)];
        let payload = Payload::Query(&query);
        let bytes = self.universal_request(url, Method::GET, payload).await?;
        let res: Data<Vec<Homework>> = Res::parse(&bytes)?;
        Ok(res.0)
    }

    /// Query homework detail
    pub async fn query_homework_detail(&self, hw: &Homework) -> crate::Result<HomeworkDetail> {
        let url = "https://spoc.buaa.edu.cn/spocnewht/kczy/queryKczyInfoByid";
        let query = [("id", &hw.id)];
        let payload = Payload::Query(&query);
        let bytes = self.universal_request(url, Method::GET, payload).await?;
        let res: HomeworkDetail = Res::parse(&bytes)?;
        Ok(res)
    }
}
