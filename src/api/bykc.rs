//! Boya Course API
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use time::{format_description, PrimitiveDateTime};

use crate::{crypto, utils, Session, SessionError};

#[derive(Deserialize)]
struct BoyaCourses {
    data: BoyaData,
}

#[derive(Deserialize)]
struct BoyaData {
    content: Vec<BoyaCourse>,
}

#[cfg_attr(feature = "table", derive(tabled::Tabled))]
#[derive(Debug, Deserialize)]
pub struct BoyaCourse {
    // 课程 ID
    pub id: u32,
    // 课程名
    #[serde(rename = "courseName")]
    #[cfg_attr(feature = "table", tabled(display_with = "tabled_boya_name"))]
    pub name: String,
    // 地点
    #[serde(rename = "coursePosition")]
    #[cfg_attr(feature = "table", tabled(display_with = "tabled_boya_position"))]
    pub position: String,
    // 开始结束和预选时间
    #[serde(flatten)]
    #[cfg_attr(feature = "table", tabled(display_with = "tabled_boya_time"))]
    pub time: BoyaTime,
    #[serde(deserialize_with = "deserialize_boya_kind")]
    #[serde(rename = "courseNewKind2")]
    #[cfg_attr(feature = "table", tabled(display_with = "tabled_boya_kind"))]
    // 课程种类
    pub kind: BoyaKind,
    #[serde(flatten)]
    #[cfg_attr(feature = "table", tabled(display_with = "tabled_boya_capacity"))]
    pub capacity: BoyaCapacity,
    // 开设校区
    #[serde(deserialize_with = "deserialize_boya_campus")]
    #[serde(rename = "courseCampus")]
    #[cfg_attr(feature = "table", tabled(display_with = "tabled_boya_campus"))]
    pub campus: BoyaCampus,
    pub selected: bool,
}

#[cfg(feature = "table")]
fn tabled_boya_name(s: &str) -> String {
    textwrap::wrap(s, 18).join("\n")
}

#[cfg(feature = "table")]
fn tabled_boya_position(s: &str) -> String {
    textwrap::wrap(s, 15).join("\n")
}

#[derive(Debug, Deserialize)]
pub struct BoyaTime {
    #[serde(deserialize_with = "deserialize_boya_time")]
    #[serde(rename = "courseStartDate")]
    pub course_start: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_boya_time")]
    #[serde(rename = "courseEndDate")]
    pub course_end: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_boya_time")]
    #[serde(rename = "courseSelectStartDate")]
    pub select_start: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_boya_time")]
    #[serde(rename = "courseSelectEndDate")]
    pub select_end: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_boya_time")]
    #[serde(rename = "courseCancelEndDate")]
    pub cancel_end: PrimitiveDateTime,
}

fn deserialize_boya_time<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let format_string =
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

    let s: String = Deserialize::deserialize(deserializer)?;

    PrimitiveDateTime::parse(&s, &format_string).map_err(|e| serde::de::Error::custom(e))
}

#[cfg(feature = "table")]
fn tabled_boya_time(time: &BoyaTime) -> String {
    let format_string = format_description::parse("[year].[month].[day] [hour]:[minute]").unwrap();

    let formatted_course_start = time.course_start.format(&format_string).unwrap();
    let formatted_course_end = time.course_end.format(&format_string).unwrap();
    let formatted_select_start = time.select_start.format(&format_string).unwrap();
    let formatted_select_end = time.select_end.format(&format_string).unwrap();

    format!(
        "             CourseTime\n{} - {}\n             SelectTime\n{} - {}",
        formatted_course_start, formatted_course_end, formatted_select_start, formatted_select_end
    )
}

#[derive(Debug, Deserialize)]
pub enum BoyaKind {
    AnQuan,
    DeYu,
    LaoDong,
    MeiYu,
    Other,
}

fn deserialize_boya_kind<'de, D>(deserializer: D) -> Result<BoyaKind, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    match value.get("kindName").and_then(Value::as_str) {
        Some(kind_name) => match kind_name {
            "安全教育" => Ok(BoyaKind::AnQuan),
            "德育" => Ok(BoyaKind::DeYu),
            "劳动教育" => Ok(BoyaKind::LaoDong),
            "美育" => Ok(BoyaKind::MeiYu),
            _ => Ok(BoyaKind::Other),
        },
        None => Err(serde::de::Error::custom("missing field `kindName`")),
    }
}

#[cfg(feature = "table")]
fn tabled_boya_kind(capacity: &BoyaKind) -> String {
    match capacity {
        BoyaKind::AnQuan => "安全教育".to_string(),
        BoyaKind::DeYu => "德育".to_string(),
        BoyaKind::LaoDong => "劳动教育".to_string(),
        BoyaKind::MeiYu => "美育".to_string(),
        BoyaKind::Other => "其他".to_string(),
    }
}

#[derive(Debug, Deserialize)]
pub struct BoyaCapacity {
    #[serde(rename = "courseMaxCount")]
    pub max: u32,
    #[serde(rename = "courseCurrentCount")]
    pub current: u32,
}

#[cfg(feature = "table")]
fn tabled_boya_capacity(capacity: &BoyaCapacity) -> String {
    format!("{} / {}", capacity.current, capacity.max)
}

#[derive(Debug, Deserialize)]
pub enum BoyaCampus {
    XueYuanLu,
    ShaHe,
    All,
    Other,
}

fn deserialize_boya_campus<'de, D>(deserializer: D) -> Result<BoyaCampus, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    match value {
        "[1]" => Ok(BoyaCampus::XueYuanLu),
        "[2]" => Ok(BoyaCampus::ShaHe),
        "ALL" => Ok(BoyaCampus::All),
        _ => Ok(BoyaCampus::Other),
    }
}

#[cfg(feature = "table")]
fn tabled_boya_campus(capacity: &BoyaCampus) -> String {
    match capacity {
        BoyaCampus::XueYuanLu => "学院路".to_string(),
        BoyaCampus::ShaHe => "沙河".to_string(),
        BoyaCampus::All => "全部".to_string(),
        BoyaCampus::Other => "其他".to_string(),
    }
}

impl Session {
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
        let lable = match url.find("token=") {
            Some(s) => s,
            None => return Err(SessionError::NoToken("From Boya Login".to_string())),
        };
        let start = lable + "token=".len();
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
    pub async fn bykc_universal_request(
        &self,
        query: &str,
        url: &str,
        token: &str,
    ) -> Result<String, SessionError> {
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
        header.insert(
            HeaderName::from_bytes(b"Ak").unwrap(),
            HeaderValue::from_str(&ak).unwrap(),
        );
        header.insert(
            HeaderName::from_bytes(b"Auth_token").unwrap(),
            HeaderValue::from_str(token).unwrap(),
        );
        header.insert(
            HeaderName::from_bytes(b"Authtoken").unwrap(),
            HeaderValue::from_str(token).unwrap(),
        );
        header.insert(
            HeaderName::from_bytes(b"Sk").unwrap(),
            HeaderValue::from_str(&sk).unwrap(),
        );
        header.insert(
            HeaderName::from_bytes(b"Ts").unwrap(),
            HeaderValue::from_str(&time.to_string()).unwrap(),
        );

        // 获取 JSESSIONID
        let res = self.post(url).headers(header).json(&body).send().await?;
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
        let res = serde_json::from_str::<BoyaCourses>(&res)?;
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
    println!("{:?}", res);
    // println!("{}", utils::table(res));

    session.save();
}

#[test]
fn test_time() {
    // 获取当前的 UTC 时间
    let now_utc = time::OffsetDateTime::now_utc();

    // 获取本地时区偏移量
    let local_offset = time::UtcOffset::from_hms(8, 0, 0).unwrap();

    // 将 UTC 时间转换为本地时间
    let now_local = now_utc.to_offset(local_offset);

    // 将 OffsetDateTime 转换为 PrimitiveDateTime
    let now = PrimitiveDateTime::new(now_local.date(), now_local.time());
    println!("{}", now);
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

#[test]
fn serde_datetime() {
    #[derive(Deserialize, Debug)]
    struct MyStruct {
        #[serde(deserialize_with = "deserialize_datetime")]
        date: PrimitiveDateTime,
    }

    // 自定义反序列化函数
    fn deserialize_datetime<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 定义自定义格式字符串
        let format_string =
            format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

        let s: String = Deserialize::deserialize(deserializer)?;
        println!("Deserializing date string: {}", s); // 添加调试信息

        PrimitiveDateTime::parse(&s, &format_string).map_err(|e| {
            println!("Error parsing date string: {}", e); // 添加调试信息
            serde::de::Error::custom(e)
        })
    }

    let json_data = r#"{"date": "2024-11-30 14:30:00"}"#;

    match serde_json::from_str::<MyStruct>(json_data) {
        Ok(my_struct) => println!("Deserialized date: {:?}", my_struct.date),
        Err(e) => println!("Error deserializing JSON: {}", e),
    }
}
