use crate::Error;
use crate::crypto::rand::{Rng, WyRng};

use super::BoyaApi;
use super::data::{
    Coordinate, Course, Data, Res, Selected, Semester, SignInfo, SignRes, Statistic,
};

impl BoyaApi {
    /// # Get Current Semester
    pub async fn get_semester(&self) -> crate::Result<Semester> {
        let url = "https://bykc.buaa.edu.cn/sscv/getAllConfig";
        let payload = serde_json::json!({});
        let bytes = self.universal_request(url, &payload).await?;
        let res: Data<Semester> = Res::parse(&bytes)?;
        Ok(res.0)
    }

    /// # Query Course List
    pub async fn query_courses(&self, page: u8, size: u8) -> crate::Result<Vec<Course>> {
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/queryStudentSemesterCourseByPage
        let url = "https://bykc.buaa.edu.cn/sscv/queryStudentSemesterCourseByPage";
        // 说真的我从来没想过一页十个是不够的. 开放选课半个月后才真的上课, 学校是真**啊
        // 不过考虑到可以通过 `query_selected` 获取那些选了但是在这里看不到的课, 所以不推荐用这个分页
        let payload = serde_json::json!({
            "pageNumber": page,
            "pageSize": size,
        });
        let bytes = self.universal_request(url, &payload).await?;
        let res: Data<Vec<Course>> = Res::parse(&bytes)?;
        Ok(res.0)
    }

    /// # Query Single Course Info
    ///
    /// - Input: Course ID from [Course] via [BoyaApi::query_courses]
    pub async fn query_course(&self, id: u32) -> crate::Result<Course> {
        let url = "https://bykc.buaa.edu.cn/sscv/queryCourseById";
        let payload = serde_json::json!({
            "id": id,
        });
        let bytes = self.universal_request(url, &payload).await?;
        let res: Course = Res::parse(&bytes)?;
        Ok(res)
    }

    /// # Query Selected Course List
    ///
    /// - Input: Semester from [Semester::estimated_current]
    pub async fn query_selected(&self, semester: Semester) -> crate::Result<Vec<Selected>> {
        let url = "https://bykc.buaa.edu.cn/sscv/queryChosenCourse";
        // 要求时间格式为 hh:mm:ss, to_string 方法会导致 秒 有小数点, 不过似乎不影响
        let payload = serde_json::json!({
            "startDate": semester.start.to_string(),
            "endDate": semester.end.to_string(),
        });
        let bytes = self.universal_request(url, &payload).await?;
        let res: Data<Vec<Selected>> = Res::parse(&bytes)?;
        Ok(res.0)
    }

    /// # Query Statistic
    pub async fn query_statistic(&self) -> crate::Result<Statistic> {
        let url = "https://bykc.buaa.edu.cn/sscv/queryStatisticByUserId";
        let payload = serde_json::json!({});
        let bytes = self.universal_request(url, &payload).await?;
        let res: Data<Statistic> = Res::parse(&bytes)?;
        Ok(res.0)
    }

    /// # Select Course
    ///
    /// - Input: Course ID from [Course] via [BoyaApi::query_course]
    pub async fn select_course(&self, id: u32) -> crate::Result<()> {
        let url = "https://bykc.buaa.edu.cn/sscv/choseCourse";
        let payload = serde_json::json!({
            "courseId": id,
        });
        // data 字段包含一个 courseCurrentCount 字段, 操作后的当前容量, 感觉没什么用
        let _ = self.universal_request(url, &payload).await?;
        Ok(())
    }

    /// # Drop Course
    ///
    /// - Input: Course ID from [Course] via [BoyaApi::query_course] or [Selected] via [BoyaApi::query_selected]
    pub async fn drop_course(&self, id: u32) -> crate::Result<()> {
        let url = "https://bykc.buaa.edu.cn/sscv/delChosenCourse";
        let payload = serde_json::json!({
            "id": id,
        });
        // data 字段包含一个 courseCurrentCount 字段, 操作后的当前容量, 感觉没什么用
        let _ = self.universal_request(url, &payload).await?;
        Ok(())
    }

    // 这个接口只在 Android UA 时才能找到, 但不妨碍使用
    /// # Sign Course (Internal)
    async fn sign_course(&self, id: u32, c: &Coordinate, t: u8) -> crate::Result<SignRes> {
        let url = "https://bykc.buaa.edu.cn/sscv/signCourseByUser";
        let mut rng = WyRng::new();
        let offset = 1e-5;

        let lng_offset = rng.random_range(-offset..offset);
        let lat_offset = rng.random_range(-offset..offset);

        // signType 1 为签到, 2 为签退
        let payload = serde_json::json!({
            "courseId": id,
            "signLat": c.latitude + lat_offset,
            "signLng": c.longitude + lng_offset,
            "signType": t,
        });
        let bytes = self.universal_request(url, &payload).await?;
        let res: Data<SignRes> = Res::parse(&bytes)?;
        Ok(res.0)
    }

    /// # Check-in Course
    ///
    /// - Input:
    ///     - Course ID from [Course]
    ///     - Coordinate from [Course.sign_config.coordinate]
    pub async fn checkin_course(&self, id: u32, c: &Coordinate) -> crate::Result<SignInfo> {
        Ok(self.sign_course(id, c, 1).await?.checkin)
    }

    /// # Check-out Course
    ///
    /// - Input:
    ///     - Course ID from [Course]
    ///     - Coordinate from [Course.sign_config.coordinate]
    pub async fn checkout_course(&self, id: u32, c: &Coordinate) -> crate::Result<SignInfo> {
        self.sign_course(id, c, 2)
            .await?
            .checkout
            .ok_or(Error::server("No checkout result").with_label("Boya"))
    }
}
