// 多亏了**学校神乎其神的后端, 这坨石山我是一点也不敢重构啊

use serde::{Deserialize, Deserializer, Serialize};

use std::borrow::Cow;

// #[derive(Deserialize)]
// pub(super) struct Res<T> {
//     pub code: String,
//     pub msg: String,
//     pub result: T,
// }

// ====================
// 用于解析需要评教的列表 Json
// ====================

#[derive(Debug, Deserialize)]
pub(super) struct _EvaluationList {
    #[serde(rename = "result")]
    pub(super) list: Vec<EvaluationListItem>,
}

#[derive(Debug, Deserialize)]
pub struct EvaluationListItem {
    // 两个看不懂的 ID
    pub(super) rwid: String,
    pub(super) wjid: String,
    // 是否已评教, 1 为是, 0 为否
    #[serde(deserialize_with = "deserialize_evaluation_state")]
    #[serde(rename = "ypjcs")]
    pub state: bool,
    // 看不懂
    pub(super) sxz: String,
    // 评教人代码
    pub(super) pjrdm: String,
    // 评教人名称
    pub(super) pjrmc: String,
    // 被评人代码
    pub(super) bpdm: String,
    // 被评人名称
    #[serde(rename = "bpmc")]
    pub teacher: String,
    // 课程代码
    pub(super) kcdm: String,
    // 课程名称
    #[serde(rename = "kcmc")]
    pub course: String,
    // 任务号???
    pub(super) rwh: String,
}

fn deserialize_evaluation_state<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value: u8 = Deserialize::deserialize(deserializer)?;
    Ok(matches!(value, 1))
}

// ====================
// 用于解析评教表单返回 Json
// ====================

#[derive(Deserialize)]
pub(super) struct _EvaluationForm {
    #[serde(deserialize_with = "deserialize_evaluation_form")]
    pub result: EvaluationForm,
}

fn deserialize_evaluation_form<'de, D>(deserializer: D) -> Result<EvaluationForm, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Vec<EvaluationForm> = Deserialize::deserialize(deserializer)?;
    value
        .into_iter()
        .next()
        .ok_or_else(|| serde::de::Error::custom("Expected at least one EvaluationForm"))
}

#[derive(Debug, Deserialize)]
pub struct EvaluationForm {
    #[serde(deserialize_with = "deserialize_evaluation_info")]
    #[serde(rename = "pjxtPjjgPjjgckb")]
    pub(super) info: EvaluationInfo,
    #[serde(deserialize_with = "deserialize_evaluation_question")]
    #[serde(rename = "pjxtWjWjbReturnEntity")]
    pub questions: Vec<EvaluationQuestion>,
    #[serde(rename = "pjmap")]
    map: EvaluationMap,
}

fn deserialize_evaluation_info<'de, D>(deserializer: D) -> Result<EvaluationInfo, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Vec<EvaluationInfo> = Deserialize::deserialize(deserializer)?;
    value
        .into_iter()
        .next()
        .ok_or_else(|| serde::de::Error::custom("Expected at least one EvaluationInfo"))
}

fn deserialize_evaluation_question<'de, D>(
    deserializer: D,
) -> Result<Vec<EvaluationQuestion>, D::Error>
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
        tasklist: Vec<EvaluationQuestion>,
    }
    let value: Wrapper = Deserialize::deserialize(deserializer)?;
    value
        .data
        .into_iter()
        .next()
        .map(|raw| raw.tasklist)
        .ok_or_else(|| serde::de::Error::custom("Expected at least one EvaluationQuestion list"))
}

#[derive(Debug, Deserialize)]
pub(super) struct EvaluationInfo {
    #[serde(rename = "pjid")]
    id1: String,
    /// 学期
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
    pub(super) rwid: String,
    pub(super) wjid: String,
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
struct EvaluationMap {
    #[serde(rename = "PJJGXXBM")]
    pj1: String,
    #[serde(rename = "RWID")]
    rwid: String,
    #[serde(rename = "PJJGBM")]
    pj2: String,
}

#[derive(Debug, Deserialize)]
pub struct EvaluationQuestion {
    /// 题目 id
    #[serde(rename = "tmid")]
    pub id: String,
    /// 题目类型
    #[serde(deserialize_with = "deserialize_evaluation_question_kind")]
    #[serde(rename = "tmlx")]
    pub is_choice: bool,
    /// 题目名称
    #[serde(rename = "tgmc")]
    pub name: String,
    /// 选项列表
    #[serde(rename = "tmxxlist")]
    pub options: Vec<EvaluationOption>,
}

fn deserialize_evaluation_question_kind<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    Ok(matches!(value, "1"))
}

#[derive(Debug, Deserialize)]
pub struct EvaluationOption {
    // 选项 id
    #[serde(rename = "tmxxid")]
    pub id: String,
    // 折算分数
    #[serde(rename = "xxfz")]
    pub score: f32,
}

// ====================
// 用于构造请求 Json
// ====================

pub enum EvaluationAnswer {
    Choice(usize),
    Completion(String),
}

impl EvaluationForm {
    pub fn fill_default<'a>(&'a self) -> EvaluationCompleted<'a> {
        // 首先, 我们获取题目数量
        let len = self.questions.len();
        // 用于计算总分
        let mut score = 0f32;
        // 新建一个固定容量的空的完成列表
        let mut completed: Vec<EvaluationCompletedQuestion> = Vec::with_capacity(len);
        // 除了第一个题目选择第二个, 其他都选第一个
        let mut choice = 1;

        for q in &self.questions {
            if q.is_choice {
                let option = q.options.get(choice).unwrap();
                score += option.score;
                let ans: Vec<Cow<'a, str>> = vec![Cow::Borrowed(option.id.as_str())];
                completed.push(EvaluationCompletedQuestion {
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
                let option = q.options.first().unwrap();
                let ans = vec![];
                completed.push(EvaluationCompletedQuestion {
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

        EvaluationCompleted::new(score, &self.map, &self.info, completed)
    }

    pub fn fill<'a>(&'a self, ans: Vec<EvaluationAnswer>) -> EvaluationCompleted<'a> {
        let question_len = self.questions.len();
        let ans_len = ans.len();
        if question_len != ans_len {
            panic!("Question count not match: {question_len} != {ans_len}");
        }
        let mut score = 0f32;
        let mut completed: Vec<EvaluationCompletedQuestion> = Vec::with_capacity(question_len);
        for (question, answer) in self.questions.iter().zip(ans.into_iter()) {
            match answer {
                EvaluationAnswer::Choice(index) => {
                    let option = question.options.get(index).unwrap();
                    score += option.score;
                    let ans: Vec<Cow<'a, str>> = vec![Cow::Borrowed(option.id.as_str())];
                    completed.push(EvaluationCompletedQuestion {
                        sjly: "1",
                        stlx: "1",
                        wjid: &self.info.wjid,
                        rwid: &self.info.rwid,
                        wjstctid: "",
                        question_id: &question.id,
                        answer: ans,
                    });
                }
                EvaluationAnswer::Completion(answer) => {
                    let option = question.options.first().unwrap();
                    let mut ans: Vec<Cow<'a, str>> = Vec::with_capacity(1);
                    if !answer.is_empty() {
                        ans.push(Cow::Owned(answer));
                    }
                    completed.push(EvaluationCompletedQuestion {
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

        EvaluationCompleted::new(score, &self.map, &self.info, completed)
    }
}

#[derive(Debug, Serialize)]
pub struct EvaluationCompleted<'a> {
    pjidlist: Vec<()>,
    #[serde(rename = "pjjglist")]
    content: Vec<EvaluationCompletedList<'a>>,
    pjzt: &'static str,
}

impl<'a> EvaluationCompleted<'a> {
    fn new(
        score: f32,
        map: &'a EvaluationMap,
        info: &'a EvaluationInfo,
        completed: Vec<EvaluationCompletedQuestion<'a>>,
    ) -> Self {
        let content: Vec<EvaluationCompletedList> = vec![EvaluationCompletedList {
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

impl<'a> EvaluationCompleted<'a> {
    pub(super) fn rwid(&self) -> &str {
        self.content[0].rwid
    }
    pub(super) fn wjid(&self) -> &str {
        self.content[0].wjid
    }
    pub fn score(&self) -> f32 {
        self.content[0].score
    }
}

#[derive(Debug, Serialize)]
struct EvaluationCompletedList<'a> {
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
    map: &'a EvaluationMap,
    #[serde(rename = "pjrdm")]
    student_id: &'a str,
    #[serde(rename = "pjrjsdm")]
    id2: &'a str,
    #[serde(rename = "pjrxm")]
    student_name: &'a str,
    pjsx: u8,
    #[serde(rename = "pjxxlist")]
    questions: Vec<EvaluationCompletedQuestion<'a>>,
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
struct EvaluationCompletedQuestion<'a> {
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
