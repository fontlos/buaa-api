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
    // 被评人代码
    pub(super) bpdm: String,
    // 课程代码
    pub(super) kcdm: String,
    // 课程名称
    #[serde(rename = "kcmc")]
    pub course: String,
    // 被评人名称
    #[serde(rename = "bpmc")]
    pub teacher: String,
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
    info: EvaluationInfo,
    pub questions: Vec<EvaluationQuestion>,
}

#[derive(Debug)]
struct EvaluationInfo {
    id: String,
}

#[derive(Debug)]
pub struct EvaluationQuestion {
    // 题目 id
    pub id: String,
    // 题目名称
    pub name: String,
    // 选项列表
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
            info: Vec<RawInfo>,
            #[serde(rename = "pjxtWjWjbReturnEntity")]
            entity: RawEntity,
        }

        #[derive(Deserialize)]
        struct RawInfo {
            #[serde(rename = "pjid")]
            id: String,
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
            .map(|raw_info| EvaluationInfo { id: raw_info.id })
            .ok_or_else(|| serde::de::Error::custom("Expected at least one info"))?;

        // 最后拿到 entity 里面的原始 list 数组, 其中包含了所有的题目的数组, 通过 flat_map 将其展开, 映射到 EvaluationQuestion
        let questions = raw_response
            .entity
            .list
            .into_iter()
            .flat_map(|raw_list| {
                raw_list
                    .tasklist
                    .into_iter()
                    .map(|raw_task| EvaluationQuestion {
                        id: raw_task.id,
                        name: raw_task.name,
                        options: raw_task
                            .optionlist
                            .into_iter()
                            .map(|raw_option| EvaluationOption {
                                op_id: raw_option.id,
                                score: raw_option.score,
                            })
                            .collect(),
                    })
            })
            .collect();

        Ok(EvaluationForm { info, questions })
    }
}

// ====================
// 这些用来构造请求 Json 的
// ====================

#[derive(Debug, Serialize)]
pub struct EvaluationCompletedForm {
    content: Vec<EvaluationCompletedList>,
}

#[derive(Debug, Serialize)]
struct EvaluationCompletedList {
    // bprdm: String,
    // kcdm: String,
    pjdf: f32,
    pjid: String,
    pjxxlist: Vec<EvaluationCompletedQuestion>,
}

#[derive(Debug, Serialize)]
struct EvaluationCompletedQuestion {
    wjid: String,
    wjssrwid: String,
    wjstid: String,
    xxdalist: Vec<String>,
}
