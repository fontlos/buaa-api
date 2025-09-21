use rand::Rng;
use serde_json::Value;
use time::Date;

use super::data::{
    _BoyaData, BoyaCoordinate, BoyaCourse, BoyaSelected, BoyaSign, BoyaSignRule, BoyaStatistic,
};

impl super::BoyaApi {
    // TODO: 考虑在这里并发的给所有 Course 获取签到信息
    /// # Query Course
    pub async fn query_course(&self) -> crate::Result<Vec<BoyaCourse>> {
        let query = serde_json::json!({
            "pageNumber": 1,
            "pageSize": 10,
        });
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryStudentSemesterCourseByPage
        let url = "https://bykc.buaa.edu.cn/sscv/queryStudentSemesterCourseByPage";
        let res: _BoyaData<Vec<BoyaCourse>> = self.universal_request(url, &query).await?;
        Ok(res.0)
    }

    // 查询签到规则, 包括签到签退的时间地点
    pub async fn query_sign_rule(&self, id: u32) -> crate::Result<Option<BoyaSignRule>> {
        let query = serde_json::json!({
            "id": id,
        });
        let url = "https://bykc.buaa.edu.cn/sscv/queryCourseById";
        let res: _BoyaData<Option<BoyaSignRule>> = self.universal_request(url, &query).await?;
        Ok(res.0)
    }

    /// # Query Selected Courses
    /// Date should like `year-month-day`
    pub async fn query_selected(&self, start: Date, end: Date) -> crate::Result<Vec<BoyaSelected>> {
        let query = serde_json::json!({
            "startDate": format!("{} 00:00:00", start),
            "endDate": format!("{} 00:00:00", end),
        });
        let url = "https://bykc.buaa.edu.cn/sscv/queryChosenCourse";
        let res: _BoyaData<Vec<BoyaSelected>> = self.universal_request(url, &query).await?;
        Ok(res.0)
    }

    /// # Query Statistic
    pub async fn query_statistic(&self) -> crate::Result<BoyaStatistic> {
        let query = serde_json::json!({});
        let url = "https://bykc.buaa.edu.cn/sscv/queryStatisticByUserId";
        let res: _BoyaData<BoyaStatistic> = self.universal_request(url, &query).await?;
        Ok(res.0)
    }

    /// # Select Course
    /// - Input: Course ID from [`query_course`](#method.query_course)
    /// - Output: Status of the request, like `{"status":"0","errmsg":"请求成功","token":null,"data":{"courseCurrentCount":340}}`
    pub async fn select_course(&self, id: u32) -> crate::Result<Value> {
        let query = serde_json::json!({
            "courseId": id,
        });
        let url = "https://bykc.buaa.edu.cn/sscv/choseCourse";
        let res: Value = self.universal_request(url, &query).await?;
        Ok(res)
    }

    /// # Drop Course
    /// - Input: Course ID from [`query_course`](#method.query_course)
    /// - Output: Status of the request, like `{"status":"0","errmsg":"请求成功","token":null,"data":{"courseCurrentCount":340}}`
    pub async fn drop_course(&self, id: u32) -> crate::Result<Value> {
        let query = serde_json::json!({
            "id": id,
        });
        let url = "https://bykc.buaa.edu.cn/sscv/delChosenCourse";
        let res: Value = self.universal_request(url, &query).await?;
        Ok(res)
    }

    // 内部方法
    // 这个接口只在 Android UA 时才能找到, 但不妨碍使用
    async fn sign_course(&self, id: u32, c: &BoyaCoordinate, t: u8) -> crate::Result<BoyaSign> {
        let mut rng = rand::rng();
        let offset = 1e-5;

        let lng_offset = rng.random_range(-offset..offset);
        let lat_offset = rng.random_range(-offset..offset);

        // signType 1 为签到, 2 为签退
        let query = serde_json::json!({
            "courseId": id,
            "signLat": c.latitude + lat_offset,
            "signLng": c.longitude + lng_offset,
            "signType": t,
        });
        let url = "https://bykc.buaa.edu.cn/sscv/signCourseByUser";
        let res: _BoyaData<BoyaSign> = self.universal_request(url, &query).await?;
        Ok(res.0)
    }

    pub async fn checkin_course(
        &self,
        id: u32,
        coordinate: &BoyaCoordinate,
    ) -> crate::Result<BoyaSign> {
        self.sign_course(id, coordinate, 1).await
    }

    pub async fn checkout_course(
        &self,
        id: u32,
        coordinate: &BoyaCoordinate,
    ) -> crate::Result<BoyaSign> {
        self.sign_course(id, coordinate, 2).await
    }
}
