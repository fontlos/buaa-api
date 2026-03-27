use crate::error::Error;
use crate::utils;
use crate::utils::time::DateTime;

use super::data::{Checkin, Course, CourseSchedule, Res, Schedule};

impl super::ClassApi {
    /// # Query one day's all schedules
    ///
    /// **Input:** DateTime
    pub async fn query_schedule(&self, date: &DateTime) -> crate::Result<Vec<Schedule>> {
        let url = "https://iclass.buaa.edu.cn:8347/app/course/get_stu_course_sched.action";
        let date = date.date();
        // YYYYMMDD
        let date_str = format!("{}{:02}{:02}", date.year(), date.month() as u8, date.day());
        let payload = [("dateStr", date_str)];
        let bytes = self.universal_request(url, &payload).await?;
        let res: Vec<Schedule> = Res::parse(&bytes)?;
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
        let url = "https://iclass.buaa.edu.cn:8347/app/choosecourse/get_myall_course.action";
        let payload = [("user_type", "1"), ("xq_code", id)];
        let bytes = self.universal_request(url, &payload).await?;
        let res: Vec<Course> = Res::parse(&bytes)?;
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
        let url = "https://iclass.buaa.edu.cn:8347/app/my/get_my_course_sign_detail.action";
        let payload = [("courseId", id)];
        let bytes = self.universal_request(url, &payload).await?;
        let res: Vec<CourseSchedule> = Res::parse(&bytes)?;
        Ok(res)
    }

    /// # Checkin schedule
    ///
    /// **Input:** Schedule ID,
    /// from [Schedule::id] via [super::ClassApi::query_schedule()] (most recommended)
    /// or [CourseSchedule::id] via [super::ClassApi::query_course_schedule()]
    pub async fn checkin(&self, id: &str) -> crate::Result<()> {
        // 2026.03.23. 签到时间现在基于服务器内部时间而非标准 UTC 了.
        // 你在干什么! 怎么敢另立标准的, 其心可诛!
        let timestamp = self.get_time().await?;
        let url = "http://iclass.buaa.edu.cn:8081/app/course/stu_scan_sign.action";
        let payload = [("courseSchedId", id), ("timestamp", &timestamp)];
        let bytes = self.universal_request(url, &payload).await?;
        let res: Checkin = Res::parse(&bytes)?;
        if res.status {
            Ok(())
        } else {
            Err(Error::server("Checkin failed").with_label("Class"))
        }
    }

    /// Calibrate the internal time of the server
    async fn get_time(&self) -> crate::Result<String> {
        let url = "http://iclass.buaa.edu.cn:8081/app/common/get_timestamp.action";
        let payload: [&str; 0] = [];
        let bytes = self.universal_request(url, &payload).await?;
        let timestamp = utils::parse_by_tag(&bytes, "\"timestamp\":", "}")
            .ok_or_else(|| Error::server("Failed to parse timestamp").with_label("Class"))?;
        Ok(timestamp.to_string())
    }
}
