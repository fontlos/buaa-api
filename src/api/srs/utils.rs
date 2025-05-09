use serde::{Deserialize, Serialize, Serializer};

// ====================
// 用于请求校验
// ====================

#[derive(Deserialize)]
pub(super) struct _SrsStatus {
    pub code: u16,
    pub msg: String,
}

// ====================
// 用于课程查询
// ====================

/// # A filter for querying courses
#[derive(Serialize)]
pub struct CourseFilter {
    // 课程查询的范围
    #[serde(rename = "teachingClassType")]
    #[serde(serialize_with = "serialize_course_range")]
    range: CourseRange,
    // 页码
    #[serde(rename = "pageNumber")]
    page: u8,
    // 每页大小
    #[serde(rename = "pageSize")]
    size: u8,
    // 校区
    campus: u8,
    // 是否显示冲突课程, 可选
    #[serde(rename = "SFCT")]
    conflict: Option<u8>,
    // 课程性质, 可选
    #[serde(rename = "KCXZ")]
    #[serde(serialize_with = "serialize_course_nature")]
    nature: Option<CourseNature>,
    // 课程类型, 可选
    #[serde(rename = "KCLB")]
    #[serde(serialize_with = "serialize_course_type")]
    r#type: Option<CourseType>,
    // 搜索关键字, 可选
    #[serde(rename = "KEY")]
    key: Option<String>,
}

impl CourseFilter {
    /// Create a default course filter
    /// # Warning
    /// - make sure the campus is correct, or you can use SrsAPI.gen_filter() to get the default campus
    pub fn new(campus: u8) -> Self {
        CourseFilter {
            range: CourseRange::SUGGEST,
            page: 1,
            size: 10,
            campus,
            conflict: Some(0),
            nature: None,
            r#type: None,
            key: None,
        }
    }

    /// Set up the range of the course query
    pub fn set_range(&mut self, range: CourseRange) {
        self.range = range;
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

    /// Display the conflict course
    pub fn set_display_conflict(&mut self) {
        self.conflict = Some(0);
    }

    /// Hide the conflict course
    pub fn set_hide_conflict(&mut self) {
        self.conflict = None;
    }

    /// Set up the nature of the course
    pub fn set_nature(&mut self, nature: Option<CourseNature>) {
        self.nature = nature;
    }

    /// Set up the type of the course
    pub fn set_type(&mut self, r#type: Option<CourseType>) {
        self.r#type = r#type;
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
pub enum CourseRange {
    /// 班级课表推荐课程
    SUGGEST,
    /// 方案内课程
    PLAN,
    /// 方案外课程
    EXTRA,
    /// 重修课程
    RETAKE,
    /// 英语课程
    English,
    /// 体育课程
    PE,
    /// 通识选修课程
    GENERAL,
    /// 科研课堂
    RESEARCH,
    /// 全校课程查询
    ALL,
}

// 序列化选课过滤器范围为对应的查询字符
fn serialize_course_range<S>(range: &CourseRange, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(course_range_to_str(range))
}

#[inline]
fn course_range_to_str(range: &CourseRange) -> &str {
    match range {
        CourseRange::SUGGEST => "TJKC",
        CourseRange::PLAN => "FANKC",
        CourseRange::EXTRA => "FAWKC",
        CourseRange::RETAKE => "CXKC",
        CourseRange::English => "YYKC",
        CourseRange::PE => "TYKC",
        CourseRange::GENERAL => "XGKC",
        CourseRange::RESEARCH => "KYKT",
        CourseRange::ALL => "ALLKC",
    }
}

/// # The nature of the course
/// Be sure to consult the corresponding notes in the document to know the specific type
pub enum CourseNature {
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
fn serialize_course_nature<S>(
    nature: &Option<CourseNature>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(n) = nature {
        match n {
            CourseNature::Compulsory => serializer.serialize_str("01"),
            CourseNature::Elective => serializer.serialize_str("02"),
            CourseNature::Limited => serializer.serialize_str("03"),
            CourseNature::Optional => serializer.serialize_str("04"),
        }
    } else {
        serializer.serialize_none()
    }
}

/// # The type of course
/// Given the letters in the order given by the school, be sure to consult the corresponding notes in the document to know the specific type
// 这抽象系统是什么惊为天人的脑回路能想出来的逆天命名方式, 字母混数字, 还有合并的, 还乱序, 简单起见直接按字母表顺序排了
// A 数学与自然科学类, B 工程基础类, C 外语类, D 思政军理类, E 体育类, FG 素质教育通识限修课, K Office Hours
// 011 数理基础课, 012 工程基础课, 013 外语课类, 021 思政课, 022 军理课, 023 体育课, 024 素质教育理论必修课
// 025 素质教育实践必修课, 026 综合素养课, 031 核心专业类, 032 一般专业类, 01 自然科学类课程
pub enum CourseType {
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
fn serialize_course_type<S>(r#type: &Option<CourseType>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(t) = r#type {
        match t {
            CourseType::A => serializer.serialize_str("A"),
            CourseType::B => serializer.serialize_str("B"),
            CourseType::C => serializer.serialize_str("C"),
            CourseType::D => serializer.serialize_str("D"),
            CourseType::E => serializer.serialize_str("E"),
            CourseType::F => serializer.serialize_str("FG"),
            CourseType::G => serializer.serialize_str("K"),
            CourseType::H => serializer.serialize_str("011"),
            CourseType::I => serializer.serialize_str("012"),
            CourseType::J => serializer.serialize_str("013"),
            CourseType::K => serializer.serialize_str("021"),
            CourseType::L => serializer.serialize_str("022"),
            CourseType::M => serializer.serialize_str("023"),
            CourseType::N => serializer.serialize_str("024"),
            CourseType::O => serializer.serialize_str("025"),
            CourseType::P => serializer.serialize_str("026"),
            CourseType::Q => serializer.serialize_str("031"),
            CourseType::R => serializer.serialize_str("032"),
            CourseType::S => serializer.serialize_str("01"),
        }
    } else {
        serializer.serialize_none()
    }
}

// ====================
// 用于课程查询
// ====================

#[derive(Deserialize)]
pub(super) struct _SrsRes1 {
    pub data: Courses,
}

#[derive(Debug, Deserialize)]
pub struct Courses {
    #[serde(rename = "total")]
    pub count: u16,
    #[serde(rename = "rows")]
    pub data: Vec<Course>,
}

#[derive(Debug, Deserialize)]
pub struct Course {
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

impl Course {
    pub fn to_opt<'a>(&'a self, filter: &'a CourseFilter) -> _SrsOpt<'a> {
        _SrsOpt {
            range: course_range_to_str(&filter.range),
            id: &self.id,
            sum: &self.sum,
        }
    }
}

// ====================
// 用于查询已选
// ====================

#[derive(Deserialize)]
pub(crate) struct _SrsRes2 {
    pub data: Vec<CourseSeleted>,
}

#[derive(Debug, Deserialize)]
pub struct CourseSeleted {
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
    can_drop: String,
    #[serde(rename = "secretVal")]
    pub sum: String,
}

impl CourseSeleted {
    pub fn can_drop(&self) -> bool {
        self.can_drop == "1"
    }

    /// # Warning!
    /// It can only be used to drop course,
    /// and you need to make sure that `can_drop()` returns true,
    /// otherwise it will fail at 'drop_course' or there will be some other unknown error
    pub fn to_opt<'a>(&'a self) -> _SrsOpt<'a> {
        _SrsOpt {
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
pub struct _SrsOpt<'a> {
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
