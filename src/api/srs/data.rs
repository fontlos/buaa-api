use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize, Serializer};

// ====================
// 用于请求
// ====================

#[derive(Deserialize)]
pub(super) struct _SrsRes<T> {
    pub code: u16,
    pub msg: String,
    pub data: T,
}

pub(super) enum _SrsBody<'a, Q: Serialize + ?Sized> {
    QueryToken,
    Form(&'a Q),
    Json(&'a Q),
    None,
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s == "1")
}

fn serialize_bool<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(if *value { "1" } else { "0" })
}

// ====================
// 用于课程查询
// ====================

// 用于提取校区的临时结构
pub(super) struct _SrsCampus(pub u8);

impl<'de> Deserialize<'de> for _SrsCampus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            student: J,
        }
        #[derive(Deserialize)]
        struct J {
            campus: String,
        }
        let i = I::deserialize(deserializer)?;
        let campus = i.student.campus.parse::<u8>().map_err(de::Error::custom)?;
        Ok(_SrsCampus(campus))
    }
}

/// # A filter for querying courses
#[derive(Serialize)]
pub struct SrsFilter {
    // 课程查询的范围
    #[serde(rename = "teachingClassType")]
    #[serde(serialize_with = "serialize_course_scope")]
    pub(super) scope: CourseScope,
    // 页码
    #[serde(rename = "pageNumber")]
    page: u8,
    // 每页大小
    #[serde(rename = "pageSize")]
    size: u8,
    // 校区
    campus: u8,
    // 是否显示冲突课程, 可选, 隐藏冲突为 0
    #[serde(rename = "SFCT")]
    #[serde(serialize_with = "serialize_bool")]
    display_conflict: bool,
    // 课程性质, 可选
    #[serde(rename = "KCXZ")]
    #[serde(serialize_with = "serialize_course_requirement")]
    requirement: Option<CourseRequirement>,
    // 课程类型, 可选
    #[serde(rename = "KCLB")]
    #[serde(serialize_with = "serialize_course_category")]
    category: Option<CourseCategory>,
    // 搜索关键字, 可选
    #[serde(rename = "KEY")]
    key: Option<String>,
}

impl SrsFilter {
    /// Create a default course filter
    /// # Warning
    /// - make sure the campus is correct, or you can use SrsAPI.gen_filter() to get the default campus
    pub fn new(campus: u8) -> Self {
        SrsFilter {
            scope: CourseScope::Suggest,
            page: 1,
            size: 10,
            campus,
            display_conflict: false,
            requirement: None,
            category: None,
            key: None,
        }
    }

    /// Set up the range of the course query
    pub fn set_range(&mut self, range: CourseScope) {
        self.scope = range;
    }

    /// Set up the page number of the course query
    pub fn set_page(&mut self, page: u8) {
        self.page = page;
    }

    /// Set up the page size of the course query
    pub fn set_size(&mut self, size: u8) {
        self.size = size;
    }

    /// # Warning, only range is RETAKE can set the campus
    /// Set up the campus as XueYuanLu
    pub fn set_campus_xueyuanlu(&mut self) {
        self.campus = 1;
    }

    /// # Warning, only range is RETAKE can set the campus
    /// Set up the campus as ShaHe
    pub fn set_campus_shahe(&mut self) {
        self.campus = 2;
    }

    /// Set display of the conflict course
    pub fn set_display_conflict(&mut self, opt: bool) {
        self.display_conflict = opt;
    }

    /// Set up the nature of the course
    pub fn set_nature(&mut self, nature: Option<CourseRequirement>) {
        self.requirement = nature;
    }

    /// Set up the type of the course
    pub fn set_type(&mut self, ctype: Option<CourseCategory>) {
        self.category = ctype;
    }

    /// Set up the key word of the course
    pub fn set_key(&mut self, key: String) {
        self.key = Some(key);
    }
}

/// # The scope of the course query
/// Be sure to consult the corresponding notes in the document to know the specific type
// 离谱首字母命名法, 甚至有一个首字母都疑似拼错了
// TJKC 班级课表推荐课程, FANKC 方案内课程, FAWKC 方案外课程, CXKC 重修课程, 只有重修课程可以选校区
// YYKC 英语课程, TYKC 体育课程, XGKC 通识选修课程, KYKT 科研课堂, ALLKC 全校课程查询
pub enum CourseScope {
    /// 班级课表推荐课程
    Suggest,
    /// 方案内课程
    WithinPlan,
    /// 方案外课程
    OutsidePlan,
    /// 重修课程
    Retake,
    /// 英语课程
    English,
    /// 体育课程
    PE,
    /// 通识选修课程
    General,
    /// 科研课堂
    Research,
    /// 全校课程查询
    All,
}

impl CourseScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            CourseScope::Suggest => "TJKC",
            CourseScope::WithinPlan => "FANKC",
            CourseScope::OutsidePlan => "FAWKC",
            CourseScope::Retake => "CXKC",
            CourseScope::English => "YYKC",
            CourseScope::PE => "TYKC",
            CourseScope::General => "XGKC",
            CourseScope::Research => "KYKT",
            CourseScope::All => "ALLKC",
        }
    }
}

// 序列化选课过滤器范围为对应的查询字符
fn serialize_course_scope<S>(scope: &CourseScope, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(scope.as_str())
}

/// # The requirement of the course
/// Be sure to consult the corresponding notes in the document to know the specific type
pub enum CourseRequirement {
    /// 必修
    Compulsory,
    /// 选修
    Elective,
    /// 限修
    Limited,
    /// 任修
    Optional,
}

// 序列化选课过滤器性质为对应的查询字符
fn serialize_course_requirement<S>(
    requirement: &Option<CourseRequirement>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(n) = requirement {
        match n {
            CourseRequirement::Compulsory => serializer.serialize_str("01"),
            CourseRequirement::Elective => serializer.serialize_str("02"),
            CourseRequirement::Limited => serializer.serialize_str("03"),
            CourseRequirement::Optional => serializer.serialize_str("04"),
        }
    } else {
        serializer.serialize_none()
    }
}

/// # The category of course
/// Given the letters in the order given by the school, be sure to consult the corresponding notes in the document to know the specific type
// 这抽象系统是什么惊为天人的脑回路能想出来的逆天命名方式, 字母混数字, 还有合并的, 还乱序, 简单起见直接按字母表顺序排了
// A 数学与自然科学类, B 工程基础类, C 外语类, D 思政军理类, E 体育类, FG 素质教育通识限修课, K Office Hours
// 011 数理基础课, 012 工程基础课, 013 外语课类, 021 思政课, 022 军理课, 023 体育课, 024 素质教育理论必修课
// 025 素质教育实践必修课, 026 综合素养课, 031 核心专业类, 032 一般专业类, 01 自然科学类课程
pub enum CourseCategory {
    /// 数学与自然科学类
    A,
    /// 工程基础类
    B,
    /// 外语类
    C,
    /// 思政军理类
    D,
    /// 体育类
    E,
    /// 素质教育通识限修课
    F,
    /// Office Hours
    G,
    /// 数理基础课
    H,
    /// 工程基础课
    I,
    /// 外语课类
    J,
    /// 思政课
    K,
    /// 军理课
    L,
    /// 体育课
    M,
    /// 素质教育理论必修课
    N,
    /// 素质教育实践必修课
    O,
    /// 综合素养课
    P,
    /// 核心专业类
    Q,
    /// 一般专业类
    R,
    /// 自然科学类课程
    S,
}

// 序列化选课过滤器类型为对应的查询字符
fn serialize_course_category<S>(
    category: &Option<CourseCategory>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(t) = category {
        match t {
            CourseCategory::A => serializer.serialize_str("A"),
            CourseCategory::B => serializer.serialize_str("B"),
            CourseCategory::C => serializer.serialize_str("C"),
            CourseCategory::D => serializer.serialize_str("D"),
            CourseCategory::E => serializer.serialize_str("E"),
            CourseCategory::F => serializer.serialize_str("FG"),
            CourseCategory::G => serializer.serialize_str("K"),
            CourseCategory::H => serializer.serialize_str("011"),
            CourseCategory::I => serializer.serialize_str("012"),
            CourseCategory::J => serializer.serialize_str("013"),
            CourseCategory::K => serializer.serialize_str("021"),
            CourseCategory::L => serializer.serialize_str("022"),
            CourseCategory::M => serializer.serialize_str("023"),
            CourseCategory::N => serializer.serialize_str("024"),
            CourseCategory::O => serializer.serialize_str("025"),
            CourseCategory::P => serializer.serialize_str("026"),
            CourseCategory::Q => serializer.serialize_str("031"),
            CourseCategory::R => serializer.serialize_str("032"),
            CourseCategory::S => serializer.serialize_str("01"),
        }
    } else {
        serializer.serialize_none()
    }
}

// ====================
// 用于课程查询
// ====================

// _SrsRes<SrsCourses>
#[derive(Debug, Deserialize)]
pub struct SrsCourses {
    #[serde(rename = "total")]
    pub count: u16,
    #[serde(rename = "rows")]
    pub data: Vec<SrsCourse>,
}

#[derive(Debug, Deserialize)]
pub struct SrsCourse {
    // 教学班 ID
    #[serde(rename = "JXBID")]
    pub(super) id: String,
    // 校区
    #[serde(rename = "XQ")]
    pub campus: String,
    // 课程代码
    #[serde(rename = "KCH")]
    pub course_code: String,
    // 课程序号
    #[serde(rename = "KXH")]
    pub course_index: String,
    #[serde(rename = "KCM")]
    pub name: String,
    // 上课时间表
    #[serde(rename = "SKSJ")]
    pub schedule: Option<Vec<CourseSchedule>>,
    // 开课单位
    #[serde(rename = "KKDW")]
    pub department: String,
    // 是否已选, 0 否
    #[serde(rename = "SFYX")]
    #[serde(deserialize_with = "deserialize_bool")]
    pub is_select: bool,
    // 学时
    #[serde(rename = "XS")]
    pub class_hours: String,
    // 学分
    #[serde(rename = "XF")]
    pub credit: String,
    // 因为学校服务器逆天设计导致 JSON 不合法, 含有重复键
    // 手动截取会出现玄学问题, 索性直接抛弃不合法的键值对
    // // 课程性质
    // #[serde(rename = "KCXZ")]
    // pub nature: String,
    // // 课程类型
    // #[serde(rename = "KCLB")]
    // pub r#type: String,
    // 教师
    #[serde(rename = "SKJSZC")]
    pub teacher: String,
    // 校验和
    #[serde(rename = "secretVal")]
    pub(super) sum: String,
    // 授课语言
    #[serde(rename = "teachingLanguageName")]
    pub lang: String,
    // 课程容量
    #[serde(rename = "internalCapacity")]
    pub internal_cap: u16,
    #[serde(rename = "internalSelectedNum")]
    pub internal_sel: u16,
    #[serde(rename = "externalCapacity")]
    pub external_cap: u16,
    #[serde(rename = "externalSelectedNum")]
    pub external_sel: u16,
}

#[derive(Debug, Deserialize)]
pub struct CourseSchedule {
    #[serde(rename = "SKZCMC")]
    pub week: String,
    #[serde(rename = "SKXQ")]
    pub weekday: String,
    #[serde(rename = "KSJC")]
    pub start_lesson: String,
    #[serde(rename = "JSJC")]
    pub end_lesson: String,
    #[serde(rename = "YPSJDD")]
    pub place: String,
}

impl SrsCourse {
    pub fn as_opt<'a>(&'a self, filter: &'a SrsFilter) -> SrsOpt<'a> {
        SrsOpt {
            range: filter.scope.as_str(),
            id: &self.id,
            sum: &self.sum,
        }
    }
}

// ====================
// 用于查询已选
// ====================

// _SrsRes<Vec<SrsSelected>>
#[derive(Debug, Deserialize)]
pub struct SrsSelected {
    #[serde(rename = "JXBID")]
    pub id: String,
    #[serde(rename = "teachingClassType")]
    pub range: Option<String>,
    #[serde(rename = "XQ")]
    pub campus: String,
    #[serde(rename = "KCH")]
    pub course_code: String,
    #[serde(rename = "KXH")]
    pub course_index: String,
    #[serde(rename = "KCM")]
    pub name: String,
    #[serde(rename = "SKJS")]
    pub teacher: String,
    #[serde(rename = "KKDW")]
    pub department: String,
    #[serde(rename = "XS")]
    pub class_hours: String,
    #[serde(rename = "XF")]
    pub credit: String,
    #[serde(rename = "SFKT")]
    #[serde(deserialize_with = "deserialize_bool")]
    pub can_drop: bool,
    #[serde(rename = "secretVal")]
    pub sum: String,
}

impl SrsSelected {
    /// # Warning!
    /// It can only be used to drop course,
    /// and you need to make sure that `can_drop` is true,
    /// otherwise it will fail at 'drop_course' or there will be some other unknown error
    pub fn as_opt<'a>(&'a self) -> SrsOpt<'a> {
        SrsOpt {
            range: self.range.as_deref().unwrap_or(""),
            id: &self.id,
            sum: &self.sum,
        }
    }
}

// ====================
// 用于选退操作
// ====================

/// # Structure for course select and drop
#[derive(Serialize)]
pub struct SrsOpt<'a> {
    // 类型
    #[serde(rename = "clazzType")]
    range: &'a str,
    // 课程 ID
    #[serde(rename = "clazzId")]
    id: &'a str,
    // 校验和
    #[serde(rename = "secretVal")]
    sum: &'a str,
}
