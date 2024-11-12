//! Boya Course API
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::{Session, SessionError, crypto, utils};

#[derive(Debug, Serialize, Deserialize)]
struct BoyaCourse{
    data: BoyaData,
}

#[derive(Debug, Serialize, Deserialize)]
struct BoyaData {
    content: Vec<Course>
}

#[derive(Debug, Serialize, Deserialize)]
struct Course {
    // 课程 ID
    id: i32,
    // 课程名
    #[serde(rename = "courseName")]
    course_name: String,
    // 地点
    #[serde(rename = "coursePosition")]
    course_position: String,
    // 开始结束和预选时间
    #[serde(rename = "courseStartDate")]
    course_start_date: String,
    #[serde(rename = "courseEndDate")]
    course_end_date: String,
    // #[serde(rename = "courseSelectType")]
    // course_select_type: i32,
    #[serde(rename = "courseSelectStartDate")]
    course_select_start_date: String,
    #[serde(rename = "courseSelectEndDate")]
    course_select_end_date: String,
    #[serde(rename = "courseCancelEndDate")]
    course_cancel_end_date: String,
    // 类型, 美育之类的
    // #[serde(rename = "courseNewKind2")]
    // course_new_kind2: CourseKind,
    #[serde(rename = "courseMaxCount")]
    course_max_count: i32,
    #[serde(rename = "courseCurrentCount")]
    course_current_count: i32,
    // // 开设校区
    // #[serde(rename = "courseCampus")]
    // course_campus: String,
    // // 开设学院
    // #[serde(rename = "courseCollege")]
    // course_college: String,
    // // 开设年级
    // #[serde(rename = "courseTerm")]
    // course_term: String,
    // // 开设人群
    // #[serde(rename = "courseGroup")]
    // course_group: String,
    // 课程作业, 至于课程作业的时间, 那没有显示的必要, 有必要直接再去查吧
    // #[serde(rename = "courseHomework")]
    // course_homework: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CourseKind {
    #[serde(rename = "kindName")]
    kind_name: String,
}

impl Session{
    /// bykc Login</br>
    /// return auth token
    pub async fn bykc_login(&self) -> Result<String, SessionError> {
        // 获取 JSESSIONID
        let res = self
            .get("https://sso.buaa.edu.cn/login?noAutoRedirect=true&service=https%3A%2F%2Fbykc.buaa.edu.cn%2Fsscv%2Fcas%2Flogin")
            .send()
            .await?;
        let url = res.url().as_str();
        let start = url.find("token=").unwrap() + "token=".len();
        let token = &url[start..];
        Ok(token.to_string())
    }

    // auth_token authtoken 为 bykc_login 获取的 token
    pub async fn bykc_query_all_course(&self, token: &str) -> Result<String, SessionError> {
        // 首先初始化 RSA, 设置公钥
        // 这是查询参数, 然后被 sha1 处理
        let query = "{\"pageNumber\":1,\"pageSize\":20}";
        // 因为查询参数不变, 值就固定 ae84e1f05f40b2e03f933450ca7a344efc0bcee6
        let sha1_query = crypto::hash::sha1(query);
        // sk参数, rsa sha1_query
        let sk = crypto::rsa(&sha1_query);
        // TODO 十六位随机字符, 这里先用固定的
        let aes_key = "SenQBA8xn6CQGNJs";
        // ak参数, rsa aes_key
        let ak = crypto::rsa(aes_key);
        // 这是请求的负载, 是使用 aes 加密的查询参数
        let body = crypto::aes::aes_encrypt(query, aes_key);
        let time = utils::get_time();

        let mut header = HeaderMap::new();
        header.insert(HeaderName::from_bytes(b"Ak").unwrap(), HeaderValue::from_str(&ak).unwrap());
        header.insert(HeaderName::from_bytes(b"Auth_token").unwrap(), HeaderValue::from_str(token).unwrap());
        header.insert(HeaderName::from_bytes(b"Authtoken").unwrap(), HeaderValue::from_str(token).unwrap());
        header.insert(HeaderName::from_bytes(b"Sk").unwrap(), HeaderValue::from_str(&sk).unwrap());
        header.insert(HeaderName::from_bytes(b"Ts").unwrap(), HeaderValue::from_str(&time.to_string()).unwrap());

        // 获取 JSESSIONID
        let res = self.post("https://bykc.buaa.edu.cn/sscv/queryStudentSemesterCourseByPage")
            .headers(header)
            .json(&body)
            .send()
            .await?;
        let res = res.text().await?;
        let res = res.trim_matches('"');
        let res = crypto::aes::aes_decrypt(&res, "SenQBA8xn6CQGNJs");
        Ok(res)
    }

    // sscv/choseCourse
}

#[tokio::test]
async fn test_bykc_login_and_query() {
    use std::time::Instant;

    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.login(&username, &password).await.unwrap();

    let token = session.bykc_login().await.unwrap();
    let res = session.bykc_query_all_course(&token).await.unwrap();

    let start = Instant::now();
    let json = serde_json::from_str::<BoyaCourse>(&res).unwrap();
    println!("Time: {:?}", start.elapsed());
    println!("{:?}", json.data.content[0]);

    session.save();
}
