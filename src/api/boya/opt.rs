use rand::Rng;
use time::Date;

use super::BoyaApi;
use super::data::{Coordinate, Course, Data, Selected, SignRes, SignRule, Statistic};

impl BoyaApi {
    // TODO: 考虑在这里并发的给所有 Course 获取签到信息
    /// # Query Course
    pub async fn query_course(&self) -> crate::Result<Vec<Course>> {
        let query = serde_json::json!({
            "pageNumber": 1,
            "pageSize": 10,
        });
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryStudentSemesterCourseByPage
        let url = "https://bykc.buaa.edu.cn/sscv/queryStudentSemesterCourseByPage";
        let res: Data<Vec<Course>> = self.universal_request(url, &query).await?;
        Ok(res.0)
    }

    /// # Query Sign Rule
    ///
    /// - Input: Course ID from [Course] via [BoyaApi::query_course]
    ///
    /// If return `Some`, this means you can sign in this course via [BoyaApi::checkin_course] and [BoyaApi::checkout_course].
    pub async fn query_sign_rule(&self, id: u32) -> crate::Result<Option<SignRule>> {
        let query = serde_json::json!({
            "id": id,
        });
        let url = "https://bykc.buaa.edu.cn/sscv/queryCourseById";
        let res: Data<Option<SignRule>> = self.universal_request(url, &query).await?;
        Ok(res.0)
    }

    /// # Query Selected Courses
    ///
    /// - Input: Start and end date.
    pub async fn query_selected(&self, start: Date, end: Date) -> crate::Result<Vec<Selected>> {
        let query = serde_json::json!({
            "startDate": format!("{} 00:00:00", start),
            "endDate": format!("{} 00:00:00", end),
        });
        let url = "https://bykc.buaa.edu.cn/sscv/queryChosenCourse";
        let res: Data<Vec<Selected>> = self.universal_request(url, &query).await?;
        Ok(res.0)
    }

    /// # Query Statistic
    pub async fn query_statistic(&self) -> crate::Result<Statistic> {
        let query = serde_json::json!({});
        let url = "https://bykc.buaa.edu.cn/sscv/queryStatisticByUserId";
        let res: Data<Statistic> = self.universal_request(url, &query).await?;
        Ok(res.0)
    }

    /// # Select Course
    ///
    /// - Input: Course ID from [Course] via [BoyaApi::query_course]
    pub async fn select_course(&self, id: u32) -> crate::Result<()> {
        let query = serde_json::json!({
            "courseId": id,
        });
        let url = "https://bykc.buaa.edu.cn/sscv/choseCourse";
        // data 字段包含一个 courseCurrentCount 字段, 操作后的当前容量, 感觉没什么用
        let _: () = self.universal_request(url, &query).await?;
        Ok(())
    }

    /// # Drop Course
    ///
    /// - Input: Course ID from [Course] via [BoyaApi::query_course] or [Selected] via [BoyaApi::query_selected]
    pub async fn drop_course(&self, id: u32) -> crate::Result<()> {
        let query = serde_json::json!({
            "id": id,
        });
        let url = "https://bykc.buaa.edu.cn/sscv/delChosenCourse";
        // data 字段包含一个 courseCurrentCount 字段, 操作后的当前容量, 感觉没什么用
        let _: () = self.universal_request(url, &query).await?;
        Ok(())
    }

    // 这个接口只在 Android UA 时才能找到, 但不妨碍使用
    /// # Sign Course (Internal)
    async fn sign_course(&self, id: u32, c: &Coordinate, t: u8) -> crate::Result<SignRes> {
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
        let res: Data<SignRes> = self.universal_request(url, &query).await?;
        Ok(res.0)
    }

    /// # Check-in Course
    ///
    /// - Input:
    ///     - Course ID from [Course] via [BoyaApi::query_course]
    ///     - Coordinate from [SignRule::coordinate] via [BoyaApi::query_sign_rule]
    pub async fn checkin_course(&self, id: u32, coordinate: &Coordinate) -> crate::Result<SignRes> {
        self.sign_course(id, coordinate, 1).await
    }

    /// # Check-out Course
    ///
    /// - Input:
    ///     - Course ID from [Course] via [BoyaApi::query_course]
    ///     - Coordinate from [SignRule::coordinate] via [BoyaApi::query_sign_rule]
    pub async fn checkout_course(
        &self,
        id: u32,
        coordinate: &Coordinate,
    ) -> crate::Result<SignRes> {
        self.sign_course(id, coordinate, 2).await
    }
}
