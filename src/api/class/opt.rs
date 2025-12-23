use serde_json::Value;

use crate::utils;

use super::data::{Course, CourseSchedule, Schedule};

impl super::ClassApi {
    /// # Query one day's all schedules
    ///
    /// **Input:** Date string, format `YYYYMMDD`,
    pub async fn query_schedule(&self, date: &str) -> crate::Result<Vec<Schedule>> {
        let url = "https://iclass.buaa.edu.cn:8346/app/course/get_stu_course_sched.action";
        let payload = [("dateStr", date)];
        let res: Vec<Schedule> = self.universal_request(url, &payload).await?;
        Ok(res)
    }

    /// # Query all course of a term
    ///
    /// **Input:** Term ID
    ///
    /// ## Example
    ///
    /// `202320242` is 2024 spring term,
    /// `202420251` is 2024 autumn term
    ///
    /// ---
    ///
    /// **Note**:
    /// If your course has parallel classes,
    /// then all of these parallel class courses may appear in your course list.
    /// Although they have different IDs, the queried `CourseSchedule` is the same.
    /// So you better check the status before signing in to avoid the timestamp being overwritten.
    pub async fn query_course(&self, id: &str) -> crate::Result<Vec<Course>> {
        let url = "https://iclass.buaa.edu.cn:8346/app/choosecourse/get_myall_course.action";
        let payload = [("user_type", "1"), ("xq_code", id)];
        let res: Vec<Course> = self.universal_request(url, &payload).await?;
        // 需要过滤掉 teacher 为空的字段, 那可能是错误的课程
        let filtered = res
            .into_iter()
            .filter(|course| !course.teacher.is_empty())
            .collect();
        Ok(filtered)
    }

    /// # Query one course's all schedules
    ///
    /// **Input:** Course ID,
    /// from [Course::id] via [super::ClassApi::query_course()]
    /// or [Schedule::course_id] via [super::ClassApi::query_schedule()]
    pub async fn query_course_schedule(&self, id: &str) -> crate::Result<Vec<CourseSchedule>> {
        let url = "https://iclass.buaa.edu.cn:8346/app/my/get_my_course_sign_detail.action";
        let payload = [("courseId", id)];
        let res: Vec<CourseSchedule> = self.universal_request(url, &payload).await?;
        Ok(res)
    }

    /// # Checkin schedule
    ///
    /// **Input:** Schedule ID,
    /// from [Schedule::id] via [super::ClassApi::query_schedule()] (most recommended)
    /// or [CourseSchedule::id] via [super::ClassApi::query_course_schedule()]
    pub async fn checkin(&self, id: &str) -> crate::Result<Value> {
        let url = "http://iclass.buaa.edu.cn:8081/app/course/stu_scan_sign.action";
        let payload = [
            ("courseSchedId", id),
            ("timestamp", &utils::get_time_millis().to_string()),
        ];
        let res: Value = self.universal_request(url, &payload).await?;
        Ok(res)
    }
}
