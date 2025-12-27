use futures::future;

use crate::{Error, utils};

use super::{_Form, _List, Completed, Form, Task};

impl super::TesApi {
    /// Get list of evaluation task
    pub async fn get_task(&self) -> crate::Result<Vec<Task>> {
        self.refresh().await?;
        let cred = self.cred.load();
        let username = cred.username()?;
        // 获取任务 ID
        let url =
            "https://spoc.buaa.edu.cn/pjxt/personnelEvaluation/listObtainPersonnelEvaluationTasks";
        // 参数说明: 我们把所有能留空的参数留空, 默认查询当前学期未评任务
        // 页相关参数只有这里刚需, 否则报错
        // 任务名称是完全没用的参数, 完全可被学年学期取代
        // 是否已评 sfyp 只在这里生效, 值为 1 时查看历史评价, 默认留空即是未评任务
        // 学年学期 xnxq(yyyy-yyyyt). 留空即是全部学期, 和上面留空组合起来就是当前学期未评任务
        // 这个参数在 `fetch_task` 中更刚需. 因为假如我们查询了所有学期所有任务, 而第一条不是当前学期,
        // 同时 `fetch_task` 没有设定查询那个指定学期, 则返回空
        let query = [("yhdm", username), ("pageNum", "1"), ("pageSize", "10")];
        let res = self
            .client
            .get(url)
            .query(&query)
            .send()
            .await?
            .bytes()
            .await?;

        let task_id = utils::parse_by_tag(&res, "\"rwid\":\"", "\"")
            .ok_or_else(|| Error::server("Empty task").with_label("Tes"))?;
        let task_count = utils::parse_by_tag(&res, "\"pjsl\":", ",")
            .and_then(|s| s.parse::<usize>().ok())
            // 一个课可能有很多老师, 需要评多次, 如果没找到这个数字就默认 32
            .unwrap_or(32);

        // 获取问卷 ID
        // 已知有五种问卷 ID: 理论课, 实践课, 英语课, 体育课, 科研课堂
        let url = "https://spoc.buaa.edu.cn/pjxt/evaluationMethodSix/getQuestionnaireListToTask";
        let query = [("rwid", task_id)];
        let bytes = self
            .client
            .get(url)
            .query(&query)
            .send()
            .await?
            .bytes()
            .await?;

        // 不解析这个结构, 直接循环匹配多个问卷 ID
        let left = "\"wjid\":\"";
        let right = "\"";
        let mut form_ids = Vec::new();
        let mut start_index = 0;
        while let Some(s) = utils::parse_by_tag(&bytes[start_index..], left, right) {
            form_ids.push(s);
            // 跳过当前匹配的部分以及右标签
            start_index = s.as_ptr() as usize - bytes.as_ptr() as usize + s.len() + right.len();
        }

        let mut tasks = Vec::<Task>::with_capacity(task_count);
        // 并发处理, 这节约了大概一半的时间
        let req: Vec<_> = form_ids
            .iter()
            .map(|id| async move { self.fetch_task(id).await })
            .collect();

        let res = future::join_all(req).await;

        for r in res {
            let mut t = r?;
            tasks.append(&mut t);
        }

        Ok(tasks)
    }

    // 用于并发获取任务
    // 注意我们没必要在这里刷新权限因为它的调用者已经刷新了
    async fn fetch_task(&self, id: &str) -> crate::Result<Vec<Task>> {
        let url = "https://spoc.buaa.edu.cn/pjxt/evaluationMethodSix/getRequiredReviewsData";
        let query = [("wjid", id)];
        let res = self
            .client
            .get(url)
            .query(&query)
            .send()
            .await?
            .bytes()
            .await?;
        Ok(serde_json::from_slice::<_List>(&res)?.list)
    }

    /// Get the evaluation form
    pub async fn get_form(&self, task: &Task) -> crate::Result<Form> {
        self.refresh().await?;
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
        self.refresh().await?;
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
        Ok(res)
    }
}
