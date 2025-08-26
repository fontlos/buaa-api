use rand::Rng;
use serde_json::Value;
use time::Date;

use super::data::{
    _BoyaData, BoyaCoordinate, BoyaCourse, BoyaSelected, BoyaSign, BoyaSignRule, BoyaStatistic,
};

impl super::BoyaApi {
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
        // TODO: 查询其他时间范围也没用, 找个接口查询本学期时间范围, 封装进去无需参数算了
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

    // 不再公开, 因为本来也只是用于辅助签到和签退, 没有其他用途
    // 这个接口只在 Android UA 时才能找到, 但不妨碍使用, 在浏览器调试时可以尝试修改 UA
    // TODO: 也许我可以考虑全局使用 Android UA 避免一些痕迹
    async fn sign_course(
        &self,
        id: u32,
        coordinate: &BoyaCoordinate,
        s_type: u8,
    ) -> crate::Result<BoyaSign> {
        let mut rng = rand::rng();
        let offset = 1e-5;

        let lng_offset = rng.random_range(-offset..offset);
        let lat_offset = rng.random_range(-offset..offset);

        // signType 1 为签到, 2 为签退
        let query = serde_json::json!({
            "courseId": id,
            "signLat": coordinate.latitude + lat_offset,
            "signLng": coordinate.longitude + lng_offset,
            "signType": s_type,
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

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::utils;

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_course() {
        let context = Context::with_auth("./data");
        // 2025.5.18 15:00 我们也成功支持 SSO 的自动刷新了
        // 现在真正可以直接调用 API 无需预处理了
        // context.login().await.unwrap();

        let boya = context.boya();
        // 2025.5.17 14:00 现在至少 Boya 的 API 是支持自动刷新的
        // boya.login().await.unwrap();

        let res = boya.query_course().await.unwrap();
        println!("{:?}", res);

        context.save_auth("./data");
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_detail() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let res = boya.query_sign_rule(7882).await.unwrap();
        println!("{:?}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_selected() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let start = utils::parse_date("2024-08-26");
        let end = utils::parse_date("2024-12-29");

        let res = boya.query_selected(start, end).await.unwrap();
        println!("{:?}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_query_statistic() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let res = boya.query_statistic().await.unwrap();

        println!("{:?}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_select() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let res = boya.select_course(6637).await.unwrap();
        println!("{}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_drop() {
        let context = Context::with_auth("./data");

        let boya = context.boya();

        let res = boya.drop_course(6637).await.unwrap();
        println!("{}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_checkin_checkout() {
        let context = Context::with_auth("./data");

        let boya = context.boya();
        let id = 7774;

        let rule = boya.query_sign_rule(id).await.unwrap().unwrap();
        println!("{:?}", rule);

        let time = crate::utils::get_datatime();
        if rule.checkin_start < time && time < rule.checkin_end {
            let res = boya.checkin_course(id, &rule.coordinate).await.unwrap();
            println!("Checkin: {:?}", res);
            return;
        }

        if rule.checkout_start < time && time < rule.checkout_end {
            let res = boya.checkout_course(id, &rule.coordinate).await.unwrap();
            println!("Checkout: {:?}", res);
            return;
        }
    }
}
