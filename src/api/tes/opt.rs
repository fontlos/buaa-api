use crate::{Error, utils};

use super::{_Form, _List, Completed, Form, Task};

impl super::TesApi {
    /// Get list of evaluation task
    ///
    /// The method has made multiple requests inside it, and the speed is slow
    pub async fn get_task(&self) -> crate::Result<Vec<Task>> {
        let cred = self.cred.load();
        let username = cred.username()?;
        // 获取任务 ID
        let url =
            "https://spoc.buaa.edu.cn/pjxt/personnelEvaluation/listObtainPersonnelEvaluationTasks";
        let query = [
            ("yhdm", username),
            ("pageNum", "1"),
            ("pageSize", "10"),
            // 任务名称
            // ("rwmc", ""),
            // 是否已评
            ("sfyp", "1"),
            // 学年学期
            ("xnxq", "2024-20251"),
        ];
        let res = self
            .client
            .get(url)
            .query(&query)
            .send()
            .await?
            .bytes()
            .await?;

        let task_id = match utils::parse_by_tag(&res, "\"rwid\":\"", "\"") {
            Some(id) => id,
            None => return Err(Error::server("Empty task").with_label("Tes")),
        };

        // 获取问卷 ID
        // 已知有五种问卷 ID: 理论课, 实践课, 英语课, 体育课, 科研课堂
        let url = "https://spoc.buaa.edu.cn/pjxt/evaluationMethodSix/getQuestionnaireListToTask";
        let query = [
            ("rwid", task_id),
            // ("pageNum", "1"),
            // ("pageSize", "999"),
            // ("sfyp", "0"),
        ];
        let bytes = self
            .client
            .get(url)
            .query(&query)
            .send()
            .await?
            .bytes()
            .await?;

        // 暂时不解析这个结构, 循环匹配多个问卷 ID
        let left = "\"wjid\":\"";
        let right = "\"";
        let mut form_ids = Vec::new();
        let mut start_index = 0;
        while let Some(s) = utils::parse_by_tag(&bytes[start_index..], left, right) {
            form_ids.push(s);
            // 跳过当前匹配的部分以及右标签
            start_index = s.as_ptr() as usize - bytes.as_ptr() as usize + s.len() + right.len();
        }

        // 考虑到一个课可能有很多老师, 需要评多次, 但 30 位应该足够多数人了
        let mut list = Vec::<Task>::with_capacity(30);
        // TODO: 并发请求
        for id in form_ids {
            let url = "https://spoc.buaa.edu.cn/pjxt/evaluationMethodSix/getRequiredReviewsData";
            let query = [
                ("wjid", id),
                // ("pageNum", "1"),
                // ("pageSize", "999"),
                ("sfyp", "1"),
                ("xnxq", "2024-20251"),
            ];
            let res = self
                .client
                .get(url)
                .query(&query)
                .send()
                .await?
                .bytes()
                .await?;
            let res = serde_json::from_slice::<_List>(&res)?;
            list.extend(res.list);
        }

        Ok(list)
    }

    /// Get the evaluation form
    pub async fn get_form(&self, task: &Task) -> crate::Result<Form> {
        let url = "https://spoc.buaa.edu.cn/pjxt/evaluationMethodSix/getQuestionnaireTopic";
        let res = self
            .client
            .get(url)
            .query(&task)
            .send()
            .await?
            .bytes()
            .await?;
        let res = serde_json::from_slice::<_Form>(&res)?;
        Ok(res.result)
    }

    /// Submit the completed evaluation form
    pub async fn submit_form(&self, complete: Completed<'_>) -> crate::Result<reqwest::Response> {
        let url = "https://spoc.buaa.edu.cn/pjxt/evaluationMethodSix/submitSaveEvaluation";
        let res = self.client.post(url).json(&complete).send().await?;

        let rwid = complete.rwid();
        let wjid = complete.wjid();
        // 也许是用于验证是否提交成功的
        let url =
            "https://spoc.buaa.edu.cn/pjxt/personnelEvaluation/checkWhetherTheTaskIsEvaluable";
        let query = [("rwid", rwid), ("wjid", wjid)];
        self.client.post(url).query(&query).send().await?;
        let url = "https://spoc.buaa.edu.cn/pjxt/system/property";
        self.client.post(url).send().await?;

        // {"code":"200","msg":"成功","msg_en":"Operation is successful","result":[{"pjid":"","pjbm":"","sfnm":"1"}]} 是否匿名??
        Ok(res)
    }
}
