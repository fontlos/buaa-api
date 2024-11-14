//! Boya Course API
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;

use crate::{Session, SessionError, crypto, utils};

#[derive(Deserialize)]
struct BoyaCourses{
    data: BoyaData,
}

#[derive(Deserialize)]
struct BoyaData {
    content: Vec<BoyaCourse>
}

#[derive(Debug, Deserialize)]
pub struct BoyaCourse {
    // 课程 ID
    pub id: i32,
    // 课程名
    #[serde(rename = "courseName")]
    pub name: String,
    // 地点
    #[serde(rename = "coursePosition")]
    pub position: String,
    // 开始结束和预选时间
    #[serde(rename = "courseStartDate")]
    pub course_start: String,
    #[serde(rename = "courseEndDate")]
    pub course_end: String,
    // #[serde(rename = "courseSelectType")]
    // select_type: i32,
    #[serde(rename = "courseSelectStartDate")]
    pub select_start: String,
    #[serde(rename = "courseSelectEndDate")]
    pub select_end: String,
    #[serde(rename = "courseCancelEndDate")]
    pub cancel_end: String,
    // 类型, 美育之类的
    // #[serde(flatten)]
    // #[serde(rename = "courseNewKind2")]
    // pub kind: CourseKind,
    #[serde(rename = "courseMaxCount")]
    pub capacity: i32,
    #[serde(rename = "courseCurrentCount")]
    pub current: i32,
    // 开设校区
    #[serde(rename = "courseCampusList")]
    pub course_campus: Vec<String>,
    // 开设学院
    #[serde(rename = "courseCollegeList")]
    pub course_college: Vec<String>,
    // 开设年级
    #[serde(rename = "courseTermList")]
    pub course_term: Vec<String>,
    // 开设人群
    #[serde(rename = "courseGroupList")]
    pub course_group: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CourseKind {
    #[serde(rename = "kindName")]
    #[allow(dead_code)]
    kind_name: String,
}

impl Session{
    /// # Boya Course Login
    /// - Need: [`sso_login`](#method.sso_login)
    /// - Output: Token for Boya Course
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

    /// # Boya Universal Request API
    /// - Need: [`bykc_login`](#method.bykc_login)
    /// - Input:
    ///     - Query: JSON String for request
    ///     - URL: API URL
    ///     - Token from [`bykc_login`](#method.bykc_login)
    ///
    /// Other Boyaa APIs don't need to be implemented, if you need to, you can extend it with this generic request API, you can find JS code like the following by intercepting all XHR requests in the browser, via stack calls <br>
    /// Locate the following sections in the `app.js` with breakpoint debugging
    ///
    /// # JS Code
    ///  ```js
    /// var y = new h.default;
    /// y.setPublicKey(b);
    /// var x = c || {}
    ///   , w = JSON.stringify(x)
    ///   , k = (0,
    /// o.default)(w).toString()
    ///   , A = y.encrypt(k)
    ///   , _ = s.getRandomStr(16)
    ///   , S = y.encrypt(_)
    ///   , D = d.default.parse(_)
    ///   , E = l.default.encrypt(d.default.parse(w), D, {
    ///     iv: D,
    ///     mode: u.default,
    ///     padding: f.default
    /// }).toString()
    ///   , I = (new Date).getTime() + "";
    /// g.sk = A,
    /// g.ak = S,
    /// g.ts = I;
    /// var C = function(e) {
    ///     var t = d.default.parse(_)
    ///       , n = l.default.decrypt(e.data, t, {
    ///         iv: t,
    ///         mode: u.default,
    ///         padding: f.default
    ///     })
    ///       , i = d.default.stringify(n);
    ///     return i && (e.data = JSON.parse(i)),
    ///     e
    /// }
    /// ```
    ///
    /// You can find `Query` in `w = JSON.stringify(x)`
    ///
    /// # Example
    ///
    /// `getUserProfile` API
    /// - URL: `https://bykc.buaa.edu.cn/sscv/getUserProfile`
    /// - Query: `{}`
    pub async fn bykc_universal_request(&self, query: &str, url: &str, token: &str) -> Result<String, SessionError> {
        // 首先初始化 RSA, 设置公钥
        // 这是查询参数, 然后被 sha1 处理
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
        let res = self.post(url)
            .headers(header)
            .json(&body)
            .send()
            .await?;
        let res = res.text().await?;
        let res = res.trim_matches('"');
        let res = crypto::aes::aes_decrypt(&res, &aes_key);
        Ok(res)
    }

    /// # Boya Course Query
    /// - Need: [`bykc_login`](#method.bykc_login)
    /// - Input: Token from [`bykc_login`](#method.bykc_login)
    pub async fn bykc_course_query(&self, token: &str) -> Result<Vec<BoyaCourse>, SessionError> {
        let query = "{\"pageNumber\":1,\"pageSize\":10}";
        let url = "https://bykc.buaa.edu.cn/sscv/queryStudentSemesterCourseByPage";
        let res = self.bykc_universal_request(query, url, token).await?;
        let res = serde_json::from_str::<BoyaCourses>(&res).unwrap();
        Ok(res.data.content)
    }

    /// # Boya Select Course
    /// - Need: [`bykc_login`](#method.bykc_login)
    /// - Input:
    ///     - Course ID from [`bykc_course_query`](#method.bykc_course_query)
    ///     - Token from [`bykc_login`](#method.bykc_login)
    /// - Output: Status of the request, like `{"status":"0","errmsg":"请求成功","token":null,"data":{"courseCurrentCount":340}}`
    pub async fn bykc_course_select(&self, id: &str, token: &str) -> Result<String, SessionError> {
        let query = format!("{{\"courseId\":{}}}", id);
        let url = "https://bykc.buaa.edu.cn/sscv/choseCourse";
        let res = self.bykc_universal_request(&query, url, token).await?;
        Ok(res)
    }

    /// # Boya Drop Course
    /// - Need: [`bykc_login`](#method.bykc_login)
    /// - Input:
    ///     - Course ID from [`bykc_course_query`](#method.bykc_course_query)
    ///     - Token from [`bykc_login`](#method.bykc_login)
    /// - Output: Status of the request, like `{"status":"0","errmsg":"请求成功","token":null,"data":{"courseCurrentCount":340}}`
    pub async fn bykc_course_drop(&self, id: &str, token: &str) -> Result<String, SessionError> {
        let query = format!("{{\"id\":{}}}", id);
        let url = "https://bykc.buaa.edu.cn/sscv/delChosenCourse";
        let res = self.bykc_universal_request(&query, url, token).await?;
        Ok(res)
    }
}

#[tokio::test]
async fn test_bykc_login_and_query() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();

    let token = session.bykc_login().await.unwrap();
    let res = session.bykc_course_query(&token).await.unwrap();
    println!("{:?}", res[0]);

    session.save();
}

#[tokio::test]
async fn test_bykc_select() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();

    let token = session.bykc_login().await.unwrap();
    let res = session.bykc_course_select("6637", &token).await.unwrap();
    println!("{}", res);

    session.save();
}

#[tokio::test]
async fn test_bykc_drop() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();

    let token = session.bykc_login().await.unwrap();
    let res = session.bykc_course_drop("6637", &token).await.unwrap();
    println!("{}", res);

    session.save();
}
