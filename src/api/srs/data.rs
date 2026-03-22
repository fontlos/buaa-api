use serde::de::Deserializer;
use serde::{Deserialize, Serialize, Serializer};

use crate::api::Data;
use crate::error::Error;
use crate::utils::time::DateTime;

#[derive(Deserialize)]
pub(crate) struct Res<T> {
    code: u16,
    msg: String,
    data: T,
}

impl<'de, T: Deserialize<'de>> Res<T> {
    pub(crate) fn parse(v: &'de [u8]) -> crate::Result<T> {
        let res: Res<T> = serde_json::from_slice(&v)?;
        if res.code != 200 {
            return Err(Error::server(format!("Response: {}", res.msg)).with_label("Srs"));
        }
        Ok(res.data)
    }
}

// ====================
// 反/序列化布尔值
// ====================

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s == "1")
}

// ====================
// 用于获取配置
// ====================

/// Configuration for SrsApi
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// Campus ID
    pub campus: Campus,
    // TODO: 暂不确定预选时这个结构如何, 可能会解析错误
    /// Batch list
    #[serde(rename = "electiveBatchList")]
    pub batchs: Vec<Batch>,
}

impl<'de> Deserialize<'de> for Data<Config> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            student: Config,
        }
        let i = I::deserialize(deserializer)?;
        Ok(Data(i.student))
    }
}

/// Campus
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Campus {
    /// XueYuanLu campus
    XueYuanLu,
    /// ShaHe campus
    ShaHe,
}

// TODO: 在 Config 中要从字符串解析, 但是 Filter 要传入数字, 虽然测试结果传字符串也可以
impl<'de> Deserialize<'de> for Campus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_str() {
            "1" => Ok(Campus::XueYuanLu),
            "2" => Ok(Campus::ShaHe),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid campus value: {}",
                s
            ))),
        }
    }
}

impl Serialize for Campus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Campus::XueYuanLu => serializer.serialize_u8(1),
            Campus::ShaHe => serializer.serialize_u8(2),
        }
    }
}

/// Batch info
#[derive(Clone, Debug, Deserialize)]
pub struct Batch {
    /// Batch ID
    #[serde(rename = "code")]
    pub id: String,
    /// Batch name
    pub name: String,
    /// Whether the batch can be selected
    #[serde(rename = "canSelect")]
    #[serde(deserialize_with = "deserialize_bool")]
    pub can_select: bool,
    /// Batch start time
    #[serde(rename = "beginTime")]
    pub start: DateTime,
    /// Batch end time
    #[serde(rename = "endTime")]
    pub end: DateTime,
}

// ====================
// 用于课程查询
// ====================

/// # Filter for querying courses
#[derive(Clone, Debug, Serialize)]
pub struct Filter {
    // 课程查询的范围
    #[serde(rename = "teachingClassType")]
    #[serde(serialize_with = "serialize_scope")]
    pub(super) scope: Scope,
    // 页码
    #[serde(rename = "pageNumber")]
    page: u8,
    // 每页大小
    #[serde(rename = "pageSize")]
    size: u8,
    // 校区
    campus: Campus,
    // 是否显示冲突课程, 可选
    // 全部显示置空, 隐藏冲突为 0, 只显示冲突为 1, 理论上用不到状态 1
    #[serde(skip_serializing_if = "is_true")]
    #[serde(rename = "SFCT")]
    #[serde(serialize_with = "serialize_conflict")]
    display_conflict: bool,
    // 课程性质, 必修限修等, 可选
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "KCXZ")]
    #[serde(serialize_with = "serialize_requirement")]
    requirement: Option<Requirement>,
    // 课程类型, 可选
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "KCLB")]
    #[serde(serialize_with = "serialize_category")]
    category: Option<Category>,
    // 搜索关键字, 可选
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "KEY")]
    key: Option<String>,
}

const fn is_true(a: &bool) -> bool {
    *a
}

// display_conflict 全部显示置空, 隐藏冲突为 0, 只显示冲突为 1, 理论上用不到状态 1
// 所以为 true 时我们跳过序列化, 那么序列化函数只需考虑 false 的情况, 直接序列化为 "0" 即可
fn serialize_conflict<S>(_: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str("0")
}

impl Filter {
    /// Create a default course filter
    ///
    /// **Warning:** make sure the campus is correct,
    /// or you can use `SrsApi.get_config()` to get the current campus
    pub fn new(campus: Campus) -> Self {
        Filter {
            scope: Scope::default(),
            page: 1,
            size: 10,
            campus,
            display_conflict: false,
            requirement: None,
            category: None,
            key: None,
        }
    }

    /// Set up the scope of the course query
    pub fn set_scope(&mut self, scope: Scope) {
        self.scope = scope;
    }

    /// Set up the page number of the course query
    pub fn set_page(&mut self, page: u8) {
        self.page = page;
    }

    /// Set up the page size of the course query
    pub fn set_size(&mut self, size: u8) {
        self.size = size;
    }

    /// **Warning**: only scope is RETAKE can set the campus
    ///
    /// Set up the campus of the course query
    pub fn set_campus(&mut self, campus: Campus) {
        self.campus = campus;
    }

    /// Set display of the conflict course query
    pub fn set_display_conflict(&mut self, opt: bool) {
        self.display_conflict = opt;
    }

    /// Set up the requirement of the course query
    pub fn set_requirement(&mut self, req: Option<Requirement>) {
        self.requirement = req;
    }

    /// Set up the category of the course query
    pub fn set_category(&mut self, category: Option<Category>) {
        self.category = category;
    }

    /// Set up the key word of the course query
    pub fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }
}

// 离谱首字母命名法, 甚至有一个首字母都疑似拼错了
// TJKC 班级课表推荐课程, FANKC 方案内课程, FAWKC 方案外课程, CXKC 重修课程, 只有重修课程可以选校区
// YYKC 英语课程, TYKC 体育课程, XGKC 通识选修课程, KYKT 科研课堂, ALLKC 全校课程查询

/// # The scope of the course query
///
/// Be sure to consult the corresponding notes in the document to know the specific scope
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scope {
    /// `班级课表推荐课程`
    #[default]
    Suggest,
    /// `方案内课程`
    WithinPlan,
    /// `方案外课程`
    OutsidePlan,
    /// `重修课程`
    Retake,
    /// `英语课程`
    English,
    /// `体育课程`
    PE,
    /// `通识选修课程`
    General,
    /// `科研课堂`
    Research,
    /// `全校课程查询`
    All,
}

impl Scope {
    pub(crate) fn as_query_str(&self) -> &'static str {
        match self {
            Scope::Suggest => "TJKC",
            Scope::WithinPlan => "FANKC",
            Scope::OutsidePlan => "FAWKC",
            Scope::Retake => "CXKC",
            Scope::English => "YYKC",
            Scope::PE => "TYKC",
            Scope::General => "XGKC",
            Scope::Research => "KYKT",
            Scope::All => "ALLKC",
        }
    }
}

// 序列化选课过滤器范围为对应的查询字符
fn serialize_scope<S>(scope: &Scope, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(scope.as_query_str())
}

/// # The requirement of the course
///
/// Be sure to consult the corresponding notes in the document to know the specific type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Requirement {
    /// `必修`
    Compulsory,
    /// `选修`
    Elective,
    /// `限修`
    Limited,
    /// `任修`
    Optional,
}

// 序列化选课过滤器性质为对应的查询字符
fn serialize_requirement<S>(
    requirement: &Option<Requirement>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(n) = requirement {
        match n {
            Requirement::Compulsory => serializer.serialize_str("01"),
            Requirement::Elective => serializer.serialize_str("02"),
            Requirement::Limited => serializer.serialize_str("03"),
            Requirement::Optional => serializer.serialize_str("04"),
        }
    } else {
        serializer.serialize_none()
    }
}

// 这抽象系统是什么惊为天人的脑回路能想出来的逆天命名方式, 字母混数字, 还有合并的, 还乱序, 简单起见直接按字母表顺序排了
// A 数学与自然科学类, B 工程基础类, C 外语类, D 思政军理类, E 体育类, FG 素质教育通识限修课, K Office Hours
// 011 数理基础课, 012 工程基础课, 013 外语课类, 021 思政课, 022 军理课, 023 体育课, 024 素质教育理论必修课
// 025 素质教育实践必修课, 026 综合素养课, 031 核心专业类, 032 一般专业类, 01 自然科学类课程

/// # The category of course
///
/// Given the letters in the order given by the school, be sure to consult the corresponding notes in the document to know the specific type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    /// `数学与自然科学类`
    A,
    /// `工程基础类`
    B,
    /// `外语类`
    C,
    /// `思政军理类`
    D,
    /// `体育类`
    E,
    /// `素质教育通识限修课`
    F,
    /// `Office Hours`
    G,
    /// `数理基础课`
    H,
    /// `工程基础课`
    I,
    /// `外语课类`
    J,
    /// `思政课`
    K,
    /// `军理课`
    L,
    /// `体育课`
    M,
    /// `素质教育理论必修课`
    N,
    /// `素质教育实践必修课`
    O,
    /// `综合素养课`
    P,
    /// `核心专业类`
    Q,
    /// `一般专业类`
    R,
    /// `自然科学类课程`
    S,
}

// 序列化选课过滤器类型为对应的查询字符
fn serialize_category<S>(category: &Option<Category>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(t) = category {
        match t {
            Category::A => serializer.serialize_str("A"),
            Category::B => serializer.serialize_str("B"),
            Category::C => serializer.serialize_str("C"),
            Category::D => serializer.serialize_str("D"),
            Category::E => serializer.serialize_str("E"),
            Category::F => serializer.serialize_str("FG"),
            Category::G => serializer.serialize_str("K"),
            Category::H => serializer.serialize_str("011"),
            Category::I => serializer.serialize_str("012"),
            Category::J => serializer.serialize_str("013"),
            Category::K => serializer.serialize_str("021"),
            Category::L => serializer.serialize_str("022"),
            Category::M => serializer.serialize_str("023"),
            Category::N => serializer.serialize_str("024"),
            Category::O => serializer.serialize_str("025"),
            Category::P => serializer.serialize_str("026"),
            Category::Q => serializer.serialize_str("031"),
            Category::R => serializer.serialize_str("032"),
            Category::S => serializer.serialize_str("01"),
        }
    } else {
        serializer.serialize_none()
    }
}

// ====================
// 用于选退操作
// ====================

/// Structure for course select and drop
#[derive(Clone, Debug, Serialize)]
pub struct Opt<'a> {
    // 范围
    #[serde(rename = "clazzType")]
    scope: &'a str,
    // 课程 ID
    #[serde(rename = "clazzId")]
    id: &'a str,
    // 校验和
    #[serde(rename = "secretVal")]
    sum: &'a str,
    // 批次 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "batchId")]
    batch: Option<&'a str>,
    // 志愿序号
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "chooseVolunteer")]
    index: Option<u8>,
}

impl<'a> Opt<'a> {
    /// Set batch ID
    pub fn set_batch(&mut self, batch: &'a str) {
        self.batch = Some(batch);
    }

    /// Set index
    pub fn set_index(&mut self, index: u8) {
        self.index = Some(index);
    }
}

// ====================
// 用于课程查询
// ====================

// Res<Data<Vec<Course>>>
impl<'de> Deserialize<'de> for Data<Vec<Course>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            // total 总数不解析了
            #[serde(rename = "rows")]
            list: Vec<Course>,
        }
        let i = I::deserialize(deserializer)?;
        Ok(Data(i.list))
    }
}

/// Course info
#[derive(Clone, Debug, Deserialize)]
pub struct Course {
    // 教学班 ID
    #[serde(rename = "JXBID")]
    pub(super) id: String,
    /// Course scope
    #[serde(skip_deserializing)]
    #[serde(default)]
    pub scope: Scope,
    /// Campus
    #[serde(rename = "XQ")]
    pub campus: String,
    // 课程代码
    /// Course code
    #[serde(rename = "KCH")]
    pub course_code: String,
    // 课程序号
    /// Course index
    #[serde(rename = "KXH")]
    pub course_index: String,
    /// Course name
    #[serde(rename = "KCM")]
    pub name: String,
    // 上课时间表
    /// Course schedule
    #[serde(rename = "SKSJ")]
    pub schedule: Option<Vec<Schedule>>,
    // 开课单位
    /// Offering department
    #[serde(rename = "KKDW")]
    pub department: String,
    /// Whether the course is conflicted
    #[serde(deserialize_with = "deserialize_bool")]
    #[serde(rename = "SFCT")]
    pub is_conflict: bool,
    /// Whether the course is selected
    #[serde(rename = "SFYX")]
    #[serde(deserialize_with = "deserialize_bool")]
    pub is_select: bool,
    // 学时
    /// Class hours
    #[serde(rename = "XS")]
    pub class_hours: String,
    // 学分
    /// Credit
    #[serde(rename = "XF")]
    pub credit: String,
    // 因为学校服务器逆天设计导致 JSON 不合法, 含有重复键
    // 手动截取会出现玄学问题, 索性直接抛弃不合法的键值对
    // // 课程要求
    // #[serde(rename = "KCXZ")]
    // pub requirement: String,
    // // 课程类型
    // #[serde(rename = "KCLB")]
    // pub category: String,
    // 教师
    /// Teacher
    #[serde(rename = "SKJSZC")]
    pub teacher: String,
    // 校验和
    #[serde(rename = "secretVal")]
    pub(super) sum: String,
    // 授课语言
    /// Teaching language
    #[serde(rename = "teachingLanguageName")]
    pub lang: String,
    // 课程容量
    /// Course internal capacity
    #[serde(rename = "internalCapacity")]
    pub internal_cap: u16,
    /// Course internal selected number
    #[serde(rename = "internalSelectedNum")]
    pub internal_sel: u16,
    /// Course external capacity
    #[serde(rename = "externalCapacity")]
    pub external_cap: u16,
    /// Course external selected number
    #[serde(rename = "externalSelectedNum")]
    pub external_sel: u16,
}

/// Course schedule item
#[derive(Clone, Debug, Deserialize)]
pub struct Schedule {
    /// Week number
    #[serde(rename = "SKZCMC")]
    pub week: String,
    /// Weekday
    #[serde(rename = "SKXQ")]
    pub weekday: String,
    /// Start lesson
    #[serde(rename = "KSJC")]
    pub start_lesson: String,
    /// End lesson
    #[serde(rename = "JSJC")]
    pub end_lesson: String,
    /// Classroom
    #[serde(rename = "YPSJDD")]
    pub location: String,
}

impl Course {
    // 你*的为什么不给 Course 一个 Scope, 还得手动插入一批
    /// Convert Course to Opt for select or drop course
    pub fn as_opt(&self) -> Opt<'_> {
        Opt {
            scope: self.scope.as_query_str(),
            id: &self.id,
            sum: &self.sum,
            batch: None,
            index: None,
        }
    }
}

// ====================
// 用于查询预选
// ====================

// Res<Data<Vec<Vec<Selected>>>>
impl<'de> Deserialize<'de> for Data<Vec<Vec<Selected>>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            #[serde(rename = "tcList")]
            list: Vec<Selected>,
        }
        let i = Vec::<I>::deserialize(deserializer)?;
        Ok(Data(i.into_iter().map(|j| j.list).collect()))
    }
}

// ====================
// 用于查询已选
// ====================

// Res<Vec<Selected>>
/// Selected course item
#[derive(Clone, Debug, Deserialize)]
pub struct Selected {
    #[serde(rename = "JXBID")]
    pub(super) id: String,
    /// Course scope
    #[serde(rename = "teachingClassType")]
    pub scope: Option<String>,
    // 课程类型
    /// Course category
    #[serde(rename = "KCLB")]
    pub category: String,
    // 课程性质, 必修限修等
    /// Course requirement, compulsory, elective, etc.
    #[serde(rename = "KCXZ")]
    pub requirement: String,
    /// Campus
    #[serde(rename = "XQ")]
    pub campus: String,
    /// Course ID
    #[serde(rename = "KCH")]
    pub course_id: String,
    /// Course index
    #[serde(rename = "KXH")]
    pub course_index: String,
    /// Course name
    #[serde(rename = "KCM")]
    pub name: String,
    /// Teacher
    #[serde(rename = "SKJS")]
    pub teacher: String,
    /// Offering department
    #[serde(rename = "KKDW")]
    pub department: String,
    /// Class hours
    #[serde(rename = "XS")]
    pub class_hours: String,
    /// Credit
    #[serde(rename = "XF")]
    pub credit: String,
    /// Whether the course can be dropped
    #[serde(rename = "SFKT")]
    #[serde(deserialize_with = "deserialize_bool")]
    pub can_drop: bool,
    #[serde(rename = "secretVal")]
    pub(crate) sum: String,
}

impl Selected {
    /// # Warning!
    ///
    /// It can only be used to drop course,
    /// and you need to make sure that `can_drop` is true,
    /// otherwise it will fail at 'drop_course' or there will be some other unknown error
    pub fn as_opt(&self) -> Opt<'_> {
        Opt {
            scope: self.scope.as_deref().unwrap_or(""),
            id: &self.id,
            sum: &self.sum,
            batch: None,
            index: None,
        }
    }
}
