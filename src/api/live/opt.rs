use reqwest::Method;

use crate::api::{Data, Payload};
use crate::error::Error;
use crate::utils::time::Week;

use super::data::Schedule;

// 什么纸张司马设计每个接口返回格式都不一样啊
impl super::LiveApi {
    /// # Get Week Schedule
    pub async fn get_week_schedule(&self, week: &Week) -> crate::Result<[Vec<Schedule>; 7]> {
        // 日接口更难看, 上下午按对象存储
        // https://classroom.msa.buaa.edu.cn/courseapi/v2/course-live/get-my-course-day?day=<DATE>
        // {"code":0,"msg":"success","list":{OBJ,OBJ}}
        let url = "https://yjapi.msa.buaa.edu.cn/courseapi/v2/schedule/get-week-schedules";
        // user_id 并不重要, 来自
        // https://classroom.msa.buaa.edu.cn/consoleapi/v2/user/group-user
        // {"code":10000,"message":"操作成功","data":{}}, id 字段
        let query = [
            ("start_at", week.start.to_date2()),
            ("end_at", week.end.to_date2()),
        ];
        let payload = Payload::Query(&query);
        // {"success":true,"result":{"code":200,"msg":"", list:[]}}, 七个元素
        // success 似乎总是 true, 但 code 可能是 400
        let bytes = self.universal_request(url, Method::GET, payload).await?;
        let res = serde_json::from_slice::<Data<[Vec<Schedule>; 7]>>(&bytes).map_err(|e| {
            Error::parse("Failed to parse week schedule".to_string()).with_source(e)
        })?;
        Ok(res.0)
    }
}
