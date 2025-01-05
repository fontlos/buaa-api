use serde::{Deserialize, Deserializer, Serialize};

// ====================
// 这些用来解析需要评教的列表的 Json
// ====================

#[derive(Debug, Deserialize)]
pub struct EvaluationList {
    #[serde(rename = "result")]
    pub list: Vec<EvaluationListItem>,
}

#[derive(Debug, Deserialize)]
pub struct EvaluationListItem {
    // 两个看不懂的 ID
    pub(super) rwid: String,
    pub(super) wjid: String,
    // 是否已评教, 1 为是, 0 为否
    #[serde(deserialize_with = "deserialize_evaluation_state")]
    #[serde(rename = "ypjcs")]
    pub(super) state: bool,
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
    match value {
        1 => Ok(true),
        _ => Ok(false),
    }
}

// ====================
// 这些用来解析评教表单的返回 Json
// ====================

#[derive(Debug)]
pub struct EvaluationForm {
    pub(super) info: EvaluationInfo,
    pub questions: Vec<EvaluationQuestion>,
    map: EvaluationMap,
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

#[derive(Debug)]
pub struct EvaluationQuestion {
    /// 题目 id
    pub id: String,
    /// 题目类型
    pub is_choice: bool,
    /// 题目名称
    pub name: String,
    /// 选项列表
    pub options: Vec<EvaluationOption>,
}

#[derive(Debug)]
pub struct EvaluationOption {
    // 选项 id
    pub op_id: String,
    // 折算分数
    pub score: f32,
}

impl<'de> Deserialize<'de> for EvaluationForm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct OuterResponse {
            result: Vec<RawResponse>,
        }

        #[derive(Deserialize)]
        struct RawResponse {
            #[serde(rename = "pjxtPjjgPjjgckb")]
            info: Vec<EvaluationInfo>,
            #[serde(rename = "pjxtWjWjbReturnEntity")]
            entity: RawEntity,
            #[serde(rename = "pjmap")]
            map: EvaluationMap,
        }

        #[derive(Deserialize)]
        struct RawEntity {
            #[serde(rename = "wjzblist")]
            list: Vec<RawList>,
        }

        #[derive(Deserialize)]
        struct RawList {
            #[serde(rename = "tklist")]
            tasklist: Vec<RawTask>,
        }

        #[derive(Deserialize)]
        struct RawTask {
            // 题目 id
            #[serde(rename = "tmid")]
            id: String,
            #[serde(rename = "tmlx")]
            kind: String,
            // 题目名称
            #[serde(rename = "tgmc")]
            name: String,
            // 选项列表
            #[serde(rename = "tmxxlist")]
            optionlist: Vec<RawOption>,
        }

        #[derive(Deserialize)]
        struct RawOption {
            // 选项 id
            #[serde(rename = "tmxxid")]
            id: String,
            // 折算分数
            #[serde(rename = "xxfz")]
            score: f32,
        }

        // 第一步解析整个结构
        let outer_response = OuterResponse::deserialize(deserializer)?;
        // 获得 result 对应的数组, 其中只含一个对象
        let raw_response = outer_response
            .result
            .into_iter()
            .next()
            .ok_or_else(|| serde::de::Error::custom("Expected at least one result"))?;
        // 再拿到里面的 info 数组, 其中同样只含一个对象, 取出, 映射到 EvaluationInfo
        let info = raw_response
            .info
            .into_iter()
            .next()
            .ok_or_else(|| serde::de::Error::custom("Expected at least one info"))?;

        let map = raw_response.map;

        // 最后拿到 entity 里面的原始 list 数组, 其中包含了所有的题目的数组, 通过 flat_map 将其展开, 映射到 EvaluationQuestion
        let questions = raw_response
            .entity
            .list
            .into_iter()
            .flat_map(|raw_list| {
                raw_list.tasklist.into_iter().map(|raw_task| {
                    let kind = match raw_task.kind.as_str() {
                        "1" => true,
                        _ => false,
                    };
                    EvaluationQuestion {
                        id: raw_task.id,
                        is_choice: kind,
                        name: raw_task.name,
                        options: raw_task
                            .optionlist
                            .into_iter()
                            .map(|raw_option| EvaluationOption {
                                op_id: raw_option.id,
                                score: raw_option.score,
                            })
                            .collect(),
                    }
                })
            })
            .collect();

        Ok(EvaluationForm {
            info,
            questions,
            map,
        })
    }
}

// ====================
// 这些用来构造请求 Json 的
// ====================

pub enum EvaluationAnswer {
    Choice(usize),
    Completion(String),
}

impl EvaluationForm {
    pub fn fill(self, ans: Vec<EvaluationAnswer>) -> EvaluationCompleted {
        let question_len = self.questions.len();
        let ans_len = ans.len();
        if question_len != ans_len {
            panic!("Question count not match: {} != {}", question_len, ans_len);
        }
        let mut score = 0f32;
        let mut completed: Vec<EvaluationCompletedQuestion> = Vec::with_capacity(question_len);
        for (question, answer) in self.questions.into_iter().zip(ans.into_iter()) {
            match answer {
                EvaluationAnswer::Choice(index) => {
                    let option = question.options.get(index).unwrap();
                    score += option.score;
                    let mut ans = Vec::with_capacity(1);
                    ans.push(option.op_id.clone());
                    completed.push(EvaluationCompletedQuestion {
                        sjly: "1".to_string(),
                        stlx: "1".to_string(),
                        wjid: self.info.wjid.clone(),
                        rwid: self.info.rwid.clone(),
                        wjstctid: "".to_string(),
                        question_id: question.id,
                        answer: ans,
                    });
                }
                EvaluationAnswer::Completion(answer) => {
                    let option = question.options.get(0).unwrap();
                    let mut ans = Vec::with_capacity(1);
                    if !answer.is_empty() {
                        ans.push(answer);
                    }
                    completed.push(EvaluationCompletedQuestion {
                        sjly: "1".to_string(),
                        stlx: "6".to_string(),
                        wjid: self.info.wjid.clone(),
                        rwid: self.info.rwid.clone(),
                        wjstctid: option.op_id.clone(),
                        question_id: question.id,
                        answer: ans,
                    });
                }
            };
        }
        let mut content: Vec<EvaluationCompletedList> = Vec::with_capacity(1);
        content.push(EvaluationCompletedList {
            teacher_id: self.info.teacher_id,
            teacher_name: self.info.teacher_name,
            course_id: self.info.course_id,
            course_name: self.info.course_name,
            score,
            pjfs: "1".to_string(),
            id1: self.info.id1,
            pjlx: "2".to_string(),
            map: self.map,
            student_id: self.info.student_id,
            id2: self.info.id2.clone(),
            student_name: self.info.student_name,
            pjsx: 1,
            questions: completed,
            rwh: self.info.rwh,
            stzjid: "xx".to_string(),
            wjid: self.info.wjid,
            rwid: self.info.rwid,
            wtjjy: "".to_string(),
            xhgs: None,
            term: self.info.term,
            sfxxpj: "1".to_string(),
            sqzt: None,
            yxfz: None,
            sdrs: None,
            zsxz: self.info.id2,
            sfnm: "1".to_string(),
        });
        EvaluationCompleted {
            pjidlist: Vec::new(),
            content,
            pjzt: "1".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EvaluationCompleted {
    pjidlist: Vec<bool>,
    #[serde(rename = "pjjglist")]
    content: Vec<EvaluationCompletedList>,
    pjzt: String,
}

impl EvaluationCompleted {
    pub fn rwid(&self) -> &str {
        &self.content[0].rwid
    }
    pub fn wjid(&self) -> &str {
        &self.content[0].wjid
    }
}

#[derive(Debug, Serialize)]
struct EvaluationCompletedList {
    #[serde(rename = "bprdm")]
    teacher_id: String,
    #[serde(rename = "bprmc")]
    teacher_name: String,
    #[serde(rename = "kcdm")]
    course_id: String,
    #[serde(rename = "kcmc")]
    course_name: String,
    #[serde(rename = "pjdf")]
    score: f32,
    pjfs: String,
    #[serde(rename = "pjid")]
    id1: String,
    pjlx: String,
    #[serde(rename = "pjmap")]
    map: EvaluationMap,
    #[serde(rename = "pjrdm")]
    student_id: String,
    #[serde(rename = "pjrjsdm")]
    id2: String,
    #[serde(rename = "pjrxm")]
    student_name: String,
    pjsx: u8,
    #[serde(rename = "pjxxlist")]
    questions: Vec<EvaluationCompletedQuestion>,
    rwh: String,
    stzjid: String,
    wjid: String,
    #[serde(rename = "wjssrwid")]
    rwid: String,
    wtjjy: String,
    xhgs: Option<bool>,
    #[serde(rename = "xnxq")]
    term: String,
    sfxxpj: String,
    sqzt: Option<bool>,
    yxfz: Option<bool>,
    sdrs: Option<bool>,
    zsxz: String,
    sfnm: String,
}

#[derive(Debug, Serialize)]
struct EvaluationCompletedQuestion {
    sjly: String,
    stlx: String,
    wjid: String,
    #[serde(rename = "wjssrwid")]
    rwid: String,
    wjstctid: String,
    #[serde(rename = "wjstid")]
    question_id: String,
    // 单选题为选项 ID, 简答题为 p 标签包裹的字符串, 但懒得包了
    #[serde(rename = "xxdalist")]
    answer: Vec<String>,
}
