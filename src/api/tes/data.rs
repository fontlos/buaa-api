// 多亏了**学校神乎其神的后端, 这坨石山我是一点也不敢重构啊

use serde::{Deserialize, Deserializer, Serialize};

use std::borrow::Cow;

// #[derive(Deserialize)]
// pub(super) struct Res<T> {
//     pub code: String,
//     pub msg: String,
//     pub result: T,
// }

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value: u8 = Deserialize::deserialize(deserializer)?;
    Ok(matches!(value, 1))
}

// ====================
// 用于解析需要评教的列表 Json
// ====================

#[derive(Debug, Deserialize)]
pub(super) struct _List {
    #[serde(rename = "result")]
    pub(super) list: Vec<Task>,
}

/// Evaluation task item
#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    // 甚至不需要这个字段? **的看起来最有用的字段你不用用**字符串查询?
    // 真正的问卷 ID (可选)
    // id: String,
    // 任务 ID
    rwid: String,
    // 问卷 ID
    wjid: String,
    // 跳过这一项, 这不是查询参数
    // 是否已评
    /// Whether this evaluation is completed
    #[serde(skip_serializing)]
    #[serde(deserialize_with = "deserialize_bool")]
    #[serde(rename = "ypjcs")]
    pub state: bool,
    // 顺序号
    sxz: String,
    // 评教人代码
    pjrdm: String,
    // 评教人名称 (可选)
    // pjrmc: String,
    // 被评人代码
    bpdm: String,
    // 被评人名称
    /// Teacher
    #[serde(rename = "bpmc")]
    pub teacher: String,
    // 课程代码
    kcdm: String,
    // 课程名称
    /// Course name
    #[serde(rename = "kcmc")]
    pub course: String,
    // 课程大类名称, 但是有什么用吗
    // #[serde(skip_serializing)]
    // #[serde(rename = "kcdlmc")]
    // pub category: String,
    // 任务号
    rwh: String,
}

// ====================
// 用于解析评教表单返回 Json
// ====================

#[derive(Deserialize)]
pub(super) struct _Form {
    #[serde(deserialize_with = "deserialize_form")]
    pub result: Form,
}

fn deserialize_form<'de, D>(deserializer: D) -> Result<Form, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Vec<Form> = Deserialize::deserialize(deserializer)?;
    value
        .into_iter()
        .next()
        .ok_or_else(|| serde::de::Error::custom("Expected at least one Form"))
}

/// Evaluation form
#[derive(Debug, Deserialize)]
pub struct Form {
    // 评教系统 评教结果查看表. 这**是人能想出来的字段名啊
    #[serde(deserialize_with = "deserialize_form_info")]
    #[serde(rename = "pjxtPjjgPjjgckb")]
    info: FormInfo,
    // 评教系统 问卷返回实体. 中英混用的大**
    /// List of questions
    #[serde(deserialize_with = "deserialize_form_question")]
    #[serde(rename = "pjxtWjWjbReturnEntity")]
    pub questions: Vec<Question>,
    #[serde(rename = "pjmap")]
    map: FormMap,
}

fn deserialize_form_info<'de, D>(deserializer: D) -> Result<FormInfo, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Vec<FormInfo> = Deserialize::deserialize(deserializer)?;
    value
        .into_iter()
        .next()
        .ok_or_else(|| serde::de::Error::custom("Expected at least one FormInfo"))
}

fn deserialize_form_question<'de, D>(deserializer: D) -> Result<Vec<Question>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper {
        #[serde(rename = "wjzblist")]
        // 只有一个元素
        data: Vec<RawList>,
    }

    #[derive(Deserialize)]
    struct RawList {
        #[serde(rename = "tklist")]
        tasklist: Vec<Question>,
    }
    let value: Wrapper = Deserialize::deserialize(deserializer)?;
    value
        .data
        .into_iter()
        .next()
        .map(|raw| raw.tasklist)
        .ok_or_else(|| serde::de::Error::custom("Expected at least one FormQuestion list"))
}

#[derive(Debug, Deserialize)]
struct FormInfo {
    #[serde(rename = "pjid")]
    id1: String,
    /// 学年学期
    #[serde(rename = "xnxq")]
    term: String,
    /// 学号
    #[serde(rename = "pjrdm")]
    student_id: String,
    /// 评教人姓名
    #[serde(rename = "pjrxm")]
    student_name: String,
    #[serde(rename = "pjrjsdm")]
    id2: String,
    #[serde(rename = "wjssrwid")]
    rwid: String,
    wjid: String,
    rwh: String,
    #[serde(rename = "kcdm")]
    course_id: String,
    #[serde(rename = "kcmc")]
    course_name: String,
    #[serde(rename = "bprdm")]
    teacher_id: String,
    #[serde(rename = "bprmc")]
    teacher_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct FormMap {
    #[serde(rename = "PJJGXXBM")]
    pj1: String,
    #[serde(rename = "RWID")]
    rwid: String,
    #[serde(rename = "PJJGBM")]
    pj2: String,
}

/// Evaluation question
#[derive(Debug, Deserialize)]
pub struct Question {
    /// Question ID
    #[serde(rename = "tmid")]
    pub id: String,
    /// Question type: true for choice, false for completion
    #[serde(deserialize_with = "deserialize_bool")]
    #[serde(rename = "tmlx")]
    pub is_choice: bool,
    /// Question name
    #[serde(rename = "tgmc")]
    pub name: String,
    /// List of choices
    #[serde(rename = "tmxxlist")]
    pub choices: Vec<Choice>,
}

/// Evaluation choice
#[derive(Debug, Deserialize)]
pub struct Choice {
    /// Choice ID
    #[serde(rename = "tmxxid")]
    pub id: String,
    /// Score of this choice
    #[serde(rename = "xxfz")]
    pub score: f32,
}

// ====================
// 用于构造请求 Json
// ====================

/// Answer to a question in the evaluation form
pub enum Answer {
    /// Choice answer
    Choice(usize),
    /// Completion answer
    Completion(String),
}

impl Form {
    /// Fill the evaluation form with default answers
    pub fn fill_default<'a>(&'a self) -> Completed<'a> {
        // 首先, 我们获取题目数量
        let len = self.questions.len();
        // 用于计算总分
        let mut score = 0f32;
        // 新建一个固定容量的空的完成列表
        let mut completed: Vec<CompletedQuestion> = Vec::with_capacity(len);
        // 除了第一个题目选择第二个, 其他都选第一个
        let mut choice = 1;

        for q in &self.questions {
            if q.is_choice {
                let option = q.choices.get(choice).unwrap();
                score += option.score;
                let ans: Vec<Cow<'a, str>> = vec![Cow::Borrowed(option.id.as_str())];
                completed.push(CompletedQuestion {
                    sjly: "1",
                    stlx: "1",
                    wjid: &self.info.wjid,
                    rwid: &self.info.rwid,
                    wjstctid: "",
                    question_id: &q.id,
                    answer: ans,
                });
                choice = 0; // 只在第一个题目选择第二个选项, 其他都选择第一个
            } else {
                let option = q.choices.first().unwrap();
                let ans = vec![];
                completed.push(CompletedQuestion {
                    sjly: "1",
                    stlx: "6",
                    wjid: &self.info.wjid,
                    rwid: &self.info.rwid,
                    wjstctid: &option.id,
                    question_id: &q.id,
                    answer: ans,
                });
            }
        }

        Completed::new(score, &self.map, &self.info, completed)
    }

    /// Fill the evaluation form with given answers
    pub fn fill<'a>(&'a self, ans: Vec<Answer>) -> Completed<'a> {
        let question_len = self.questions.len();
        let ans_len = ans.len();
        if question_len != ans_len {
            panic!("Question count not match: {question_len} != {ans_len}");
        }
        let mut score = 0f32;
        let mut completed: Vec<CompletedQuestion> = Vec::with_capacity(question_len);
        for (question, answer) in self.questions.iter().zip(ans.into_iter()) {
            match answer {
                Answer::Choice(index) => {
                    let option = question.choices.get(index).unwrap();
                    score += option.score;
                    let ans: Vec<Cow<'a, str>> = vec![Cow::Borrowed(option.id.as_str())];
                    completed.push(CompletedQuestion {
                        sjly: "1",
                        stlx: "1",
                        wjid: &self.info.wjid,
                        rwid: &self.info.rwid,
                        wjstctid: "",
                        question_id: &question.id,
                        answer: ans,
                    });
                }
                Answer::Completion(answer) => {
                    let option = question.choices.first().unwrap();
                    let mut ans: Vec<Cow<'a, str>> = Vec::with_capacity(1);
                    if !answer.is_empty() {
                        ans.push(Cow::Owned(answer));
                    }
                    completed.push(CompletedQuestion {
                        sjly: "1",
                        stlx: "6",
                        wjid: &self.info.wjid,
                        rwid: &self.info.rwid,
                        wjstctid: &option.id,
                        question_id: &question.id,
                        answer: ans,
                    });
                }
            };
        }

        Completed::new(score, &self.map, &self.info, completed)
    }
}

/// The completed evaluation to be sent
#[derive(Debug, Serialize)]
pub struct Completed<'a> {
    pjidlist: Vec<()>,
    #[serde(rename = "pjjglist")]
    content: Vec<CompletedList<'a>>,
    pjzt: &'static str,
}

impl<'a> Completed<'a> {
    fn new(
        score: f32,
        map: &'a FormMap,
        info: &'a FormInfo,
        completed: Vec<CompletedQuestion<'a>>,
    ) -> Self {
        let content = vec![CompletedList {
            teacher_id: &info.teacher_id,
            teacher_name: &info.teacher_name,
            course_id: &info.course_id,
            course_name: &info.course_name,
            score,
            pjfs: "1",
            id1: &info.id1,
            pjlx: "2",
            map,
            student_id: &info.student_id,
            id2: &info.id2,
            student_name: &info.student_name,
            pjsx: 1,
            questions: completed,
            rwh: &info.rwh,
            stzjid: "xx",
            wjid: &info.wjid,
            rwid: &info.rwid,
            wtjjy: "",
            xhgs: None,
            term: &info.term,
            sfxxpj: "1",
            sqzt: None,
            yxfz: None,
            sdrs: None,
            zsxz: &info.id2,
            sfnm: "1",
        }];
        Self {
            pjidlist: Vec::new(),
            content,
            pjzt: "1",
        }
    }
}

impl<'a> Completed<'a> {
    pub(super) fn rwid(&self) -> &str {
        self.content[0].rwid
    }
    pub(super) fn wjid(&self) -> &str {
        self.content[0].wjid
    }
    /// Get the score of this evaluation
    pub fn score(&self) -> f32 {
        self.content[0].score
    }
}

#[derive(Debug, Serialize)]
struct CompletedList<'a> {
    #[serde(rename = "bprdm")]
    teacher_id: &'a str,
    #[serde(rename = "bprmc")]
    teacher_name: &'a str,
    #[serde(rename = "kcdm")]
    course_id: &'a str,
    #[serde(rename = "kcmc")]
    course_name: &'a str,
    #[serde(rename = "pjdf")]
    score: f32,
    pjfs: &'static str,
    #[serde(rename = "pjid")]
    id1: &'a str,
    pjlx: &'static str,
    #[serde(rename = "pjmap")]
    map: &'a FormMap,
    #[serde(rename = "pjrdm")]
    student_id: &'a str,
    #[serde(rename = "pjrjsdm")]
    id2: &'a str,
    #[serde(rename = "pjrxm")]
    student_name: &'a str,
    pjsx: u8,
    #[serde(rename = "pjxxlist")]
    questions: Vec<CompletedQuestion<'a>>,
    rwh: &'a str,
    stzjid: &'static str,
    wjid: &'a str,
    #[serde(rename = "wjssrwid")]
    rwid: &'a str,
    wtjjy: &'static str,
    xhgs: Option<()>,
    #[serde(rename = "xnxq")]
    term: &'a str,
    sfxxpj: &'static str,
    sqzt: Option<()>,
    yxfz: Option<()>,
    sdrs: Option<()>,
    zsxz: &'a str,
    // 是否匿名??
    sfnm: &'static str,
}

#[derive(Debug, Serialize)]
struct CompletedQuestion<'a> {
    sjly: &'static str,
    stlx: &'static str,
    wjid: &'a str,
    #[serde(rename = "wjssrwid")]
    rwid: &'a str,
    wjstctid: &'a str,
    #[serde(rename = "wjstid")]
    question_id: &'a str,
    // 单选题为选项 ID, 简答题为 p 标签包裹的字符串, 但懒得包了
    #[serde(rename = "xxdalist")]
    answer: Vec<Cow<'a, str>>,
}
