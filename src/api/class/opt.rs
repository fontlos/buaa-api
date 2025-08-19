use serde_json::Value;

use crate::utils;

use super::{_ClassCourses, _ClassSchedules, ClassCourse, ClassSchedule};

impl super::ClassApi {
    /// # Smart Classroom query all course of a term
    /// - Input: Term ID
    ///     - Example: `202320242`` is 2024 spring term, `202420251` is 2024 autumn term
    pub async fn query_course(&self, id: &str) -> crate::Result<Vec<ClassCourse>> {
        let url = "https://iclass.buaa.edu.cn:8346/app/choosecourse/get_myall_course.action";
        let query = [("user_type", "1"), ("xq_code", id)];
        let res: _ClassCourses = self.universal_request(url, &query).await?;
        Ok(res.result)
    }

    /// # Smart Classroom query one course's all schedule
    /// - Input: Course ID, from [ClassCourse]
    pub async fn query_schedule(&self, id: &str) -> crate::Result<Vec<ClassSchedule>> {
        let url = "https://iclass.buaa.edu.cn:8346/app/my/get_my_course_sign_detail.action";
        let query = [("courseId", id)];
        let res: _ClassSchedules = self.universal_request(url, &query).await?;
        Ok(res.result)
    }

    /// # Smart Classroom checkin schedule
    /// - Input: Schedule ID, from [ClassSchedule]
    pub async fn checkin(&self, id: &str) -> crate::Result<Value> {
        let url = "http://iclass.buaa.edu.cn:8081/app/course/stu_scan_sign.action";
        let query = [
            ("courseSchedId", id),
            ("timestamp", &utils::get_time_millis().to_string()),
        ];
        let res: Value = self.universal_request(url, &query).await?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[ignore]
    #[tokio::test]
    async fn test_class_query_course() {
        let context = Context::with_auth("./data");

        let class = context.class();

        let res = class.query_course("202420252").await.unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    async fn test_class_query_schedule() {
        let context = Context::with_auth("./data");

        let class = context.class();

        let res = class.query_schedule("64668").await.unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    async fn test_class_checkin() {
        let context = Context::with_auth("./data");

        let class = context.class();

        let res = class.checkin("2090542").await.unwrap();
        println!("{}", res);
    }
}
