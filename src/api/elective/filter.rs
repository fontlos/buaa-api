use serde::{Serialize, Serializer};

#[derive(Serialize)]
pub struct ElectiveFilter {
    // 课程查询的范围
    // TJKC 班级课表推荐课程, FANKC 方案内课程, FAWKC 方案外课程, CXKC 重修课程, 只有重修课程可以选校区
    // YYKC 英语课程, TYKC 体育课程, XGKC 通识选修课程, KYKT 科研课堂, ALLKC 全校课程查询
    #[serde(rename = "teachingClassType")]
    #[serde(serialize_with = "serialize_elective_range")]
    range: ElectiveRange,
    // 页码
    #[serde(rename = "pageNumber")]
    page: u8,
    // 每页大小
    #[serde(rename = "pageSize")]
    size: u8,
    // 校区, 1 学院路, 2 沙河
    campus: u8,
    // 是否显示冲突课程, 可选
    // 0 否
    #[serde(rename = "SFCT")]
    conflict: Option<u8>,
    // 课程性质, 可选
    // 01 必修, 02 选修, 03 限修, 04 任修
    #[serde(rename = "KCXZ")]
    nature: Option<String>,
    // 课程类型
    #[serde(rename = "KCLB")]
    #[serde(serialize_with = "serialize_elective_type")]
    r#type: Option<ElectiveType>,
    // 搜索关键字, 可选
    #[serde(rename = "KEY")]
    key: Option<String>,
}

impl ElectiveFilter {
    /// Create a default course filter
    pub fn new() -> Self {
        ElectiveFilter {
            range: ElectiveRange::SUGGEST,
            page: 1,
            size: 10,
            campus: 1,
            conflict: Some(0),
            nature: None,
            r#type: None,
            key: None,
        }
    }

    /// Set up the range of the course query
    pub fn set_range(&mut self, range: ElectiveRange) {
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

    /// Set up the campus as XueYuanLu
    pub fn set_campus_xueyuanlu(&mut self) {
        self.campus = 1;
    }

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
    pub fn set_nature(&mut self, nature: String) {
        self.nature = Some(nature);
    }

    /// Set up the type of the course
    pub fn set_type(&mut self, r#type: Option<ElectiveType>) {
        self.r#type = r#type;
    }

    /// Set up the key word of the course
    pub fn set_key(&mut self, key: String) {
        self.key = Some(key);
    }
}

/// # The scope of the course query
/// Be sure to consult the corresponding notes in the document to know the specific type
pub enum ElectiveRange {
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
fn serialize_elective_range<S>(range: &ElectiveRange, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match range {
        ElectiveRange::SUGGEST => serializer.serialize_str("TJKC"),
        ElectiveRange::PLAN => serializer.serialize_str("FANKC"),
        ElectiveRange::EXTRA => serializer.serialize_str("FAWKC"),
        ElectiveRange::RETAKE => serializer.serialize_str("CXKC"),
        ElectiveRange::English => serializer.serialize_str("YYKC"),
        ElectiveRange::PE => serializer.serialize_str("TYKC"),
        ElectiveRange::GENERAL => serializer.serialize_str("XGKC"),
        ElectiveRange::RESEARCH => serializer.serialize_str("KYKT"),
        ElectiveRange::ALL => serializer.serialize_str("ALLKC"),
    }
}

/// # The type of course
/// Given the letters in the order given by the school, be sure to consult the corresponding notes in the document to know the specific type
pub enum ElectiveType {
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
fn serialize_elective_type<S>(
    r#type: &Option<ElectiveType>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(t) = r#type {
        match t {
            ElectiveType::A => serializer.serialize_str("A"),
            ElectiveType::B => serializer.serialize_str("B"),
            ElectiveType::C => serializer.serialize_str("C"),
            ElectiveType::D => serializer.serialize_str("D"),
            ElectiveType::E => serializer.serialize_str("E"),
            ElectiveType::F => serializer.serialize_str("FG"),
            ElectiveType::G => serializer.serialize_str("K"),
            ElectiveType::H => serializer.serialize_str("011"),
            ElectiveType::I => serializer.serialize_str("012"),
            ElectiveType::J => serializer.serialize_str("013"),
            ElectiveType::K => serializer.serialize_str("021"),
            ElectiveType::L => serializer.serialize_str("022"),
            ElectiveType::M => serializer.serialize_str("023"),
            ElectiveType::N => serializer.serialize_str("024"),
            ElectiveType::O => serializer.serialize_str("025"),
            ElectiveType::P => serializer.serialize_str("026"),
            ElectiveType::Q => serializer.serialize_str("031"),
            ElectiveType::R => serializer.serialize_str("032"),
            ElectiveType::S => serializer.serialize_str("01"),
        }
    } else {
        serializer.serialize_none()
    }
}
