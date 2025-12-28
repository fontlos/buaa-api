// 所有人都应该来欣赏**学校神乎其神的旷世巨作, 这就是石, 重构的我想死

use serde::{Deserialize, Deserializer, Serialize};

use std::borrow::Cow;

// 当学校能整出这种**玩意时我将觉得这东西没有解析的必要了, 而且**的这个 code 一会是数字一会是字符串的
// {"code":200,"msg":null,"content":{"timestamp":"","status":500,"error":"","path":""}}
// What can I say!
// code 是没有解析的必要的, 因为类型是不可靠的, 信息是不包含的
// msg 是没有解析的必要的, 因为类型是不可靠的, 信息是不包含的.
// content 也是没有解析的必要的, 因为大概率没这个字段
// 类型不可靠 (字符串/数字混用)
// 内容不可靠 (200但实际是错误)
// 结构不可靠 (可能没有某些字段)
// 语义不可靠 (各种不一致的描述)
// 学校你**干什么吃的我***的

// 我**就只解析我要的字段, 拿不到就视为 Server Error
// "这种处理方式完全合理，对于垃圾 API 就该用简单的规则" -- DeepSeek
pub(super) struct Data<T>(pub T);

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

impl<'de> Deserialize<'de> for Data<Vec<Task>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            result: Vec<Task>,
        }
        let i = I::deserialize(deserializer)?;
        Ok(Data(i.result))
    }
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

// 从这以下是难以理解的混乱区

impl<'de> Deserialize<'de> for Data<Form> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            result: [Form; 1],
        }
        let i = I::deserialize(deserializer)?;
        let [form] = i.result;
        Ok(Data(form))
    }
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

#[derive(Debug, Deserialize, Serialize)]
struct FormInfo {
    #[serde(rename = "pjid")]
    id1: String,
    #[serde(rename = "pjrjsdm")]
    id2: String,
    rwh: String,
    #[serde(rename = "wjssrwid")]
    rwid: String,
    wjid: String,
    #[serde(rename = "xnxq")]
    term: String,
    #[serde(rename = "kcdm")]
    course_id: String,
    #[serde(rename = "kcmc")]
    course_name: String,
    #[serde(rename = "bprdm")]
    teacher_id: String,
    #[serde(rename = "bprmc")]
    teacher_name: String,
    #[serde(rename = "pjrdm")]
    student_id: String,
    #[serde(rename = "pjrxm")]
    student_name: String,
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

fn deserialize_form_question<'de, D>(deserializer: D) -> Result<Vec<Question>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct I {
        // 只有一个元素
        wjzblist: [J; 1],
    }

    #[derive(Deserialize)]
    struct J {
        // tasklist
        tklist: Vec<Question>,
    }
    let i: I = Deserialize::deserialize(deserializer)?;
    let [j] = i.wjzblist;
    Ok(j.tklist)
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
        let mut first_choice = true;
        let answers = self
            .questions
            .iter()
            .map(|q| {
                if q.is_choice {
                    let choice = if first_choice {
                        first_choice = false;
                        1
                    } else {
                        0
                    };
                    Answer::Choice(choice)
                } else {
                    Answer::Completion(String::new())
                }
            })
            .collect();

        self.fill(answers)
    }

    /// Fill the evaluation form with given answers
    pub fn fill<'a>(&'a self, ans: Vec<Answer>) -> Completed<'a> {
        let len = self.questions.len();
        if len != ans.len() {
            panic!("Question count not match");
        }

        let (score, completed) = self.questions.iter().zip(ans).fold(
            (0f32, Vec::with_capacity(len)),
            |(mut score, mut completed), (question, answer)| {
                let question_completed = match answer {
                    Answer::Choice(index) => {
                        let option = &question.choices[index];
                        score += option.score;
                        CompletedQuestion {
                            sjly: "1",
                            stlx: "1",
                            wjid: &self.info.wjid,
                            rwid: &self.info.rwid,
                            wjstctid: "",
                            question_id: &question.id,
                            answer: vec![Cow::Borrowed(option.id.as_str())],
                        }
                    }
                    Answer::Completion(text) => {
                        let option = &question.choices[0];
                        CompletedQuestion {
                            sjly: "1",
                            stlx: "6",
                            wjid: &self.info.wjid,
                            rwid: &self.info.rwid,
                            wjstctid: &option.id,
                            question_id: &question.id,
                            // 推一个空字符串进去也没什么损失, 反正分配都分配了
                            answer: vec![Cow::Owned(text)],
                        }
                    }
                };

                completed.push(question_completed);
                (score, completed)
            },
        );

        Completed::new(score, &self.info, &self.map, completed)
    }
}

/// The completed evaluation to be sent
#[derive(Debug, Serialize)]
pub struct Completed<'a> {
    pjidlist: [(); 0],
    #[serde(rename = "pjjglist")]
    content: [CompletedContent<'a>; 1],
    pjzt: &'static str,
}

impl<'a> Completed<'a> {
    fn new(
        score: f32,
        info: &'a FormInfo,
        map: &'a FormMap,
        completed: Vec<CompletedQuestion<'a>>,
    ) -> Self {
        let content = [CompletedContent {
            // 特殊的字段
            zsxz: &info.id2,
            score,
            info,
            map,
            questions: completed,
            literal: CompletedLiteral::default(),
        }];
        Self {
            pjidlist: [],
            content,
            pjzt: "1",
        }
    }
    pub(super) fn rwid(&self) -> &str {
        &self.content[0].info.rwid
    }
    pub(super) fn wjid(&self) -> &str {
        &self.content[0].info.wjid
    }
    /// Get the score of this evaluation
    pub fn score(&self) -> f32 {
        self.content[0].score
    }
}

#[derive(Debug, Serialize)]
struct CompletedContent<'a> {
    // 特殊字段, 来自 info.id2
    zsxz: &'a str,
    #[serde(rename = "pjdf")]
    score: f32,
    #[serde(flatten)]
    info: &'a FormInfo,
    #[serde(rename = "pjmap")]
    map: &'a FormMap,
    #[serde(rename = "pjxxlist")]
    questions: Vec<CompletedQuestion<'a>>,
    #[serde(flatten)]
    literal: CompletedLiteral,
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

// What can I say! **的为什么有一堆常量啊, 这玩意你不能服务器里面自己加吗
#[derive(Debug, Serialize)]
struct CompletedLiteral {
    pjsx: u8,
    pjfs: &'static str,
    pjlx: &'static str,
    stzjid: &'static str,
    wtjjy: &'static str,
    sfxxpj: &'static str,
    // 是否匿名
    sfnm: &'static str,
    xhgs: Null,
    sqzt: Null,
    yxfz: Null,
    sdrs: Null,
}

impl Default for CompletedLiteral {
    fn default() -> Self {
        Self {
            pjsx: 1,
            pjfs: "1",
            pjlx: "2",
            stzjid: "xx",
            wtjjy: "",
            sfxxpj: "1",
            sfnm: "1",
            xhgs: Null,
            sqzt: Null,
            yxfz: Null,
            sdrs: Null,
        }
    }
}

// 很难想象为什么缺少几个 null 的字段都不行
#[derive(Debug)]
struct Null;

impl Serialize for Null {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_none()
    }
}
