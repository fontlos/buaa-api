use crate::{Error, utils};

use super::{
    _EvaluationForm, _EvaluationList, EvaluationCompleted, EvaluationForm, EvaluationListItem,
};

impl super::TesApi {
    /// Get a list of the ones that need to be evaluated <br>
    /// The method has made multiple requests inside it, and the speed is slow
    pub async fn get_evaluation_list(&self) -> crate::Result<Vec<EvaluationListItem>> {
        // 考虑到 rwid 只有这一个地方用到, 所以直接在这里获取
        // 获取账号
        let cred = self.cred.load();
        let username = cred.username()?;
        // 获取 rwid
        // 省略的无用查询参数 &rwmc=&sfyp=0
        let url = format!(
            "https://spoc.buaa.edu.cn/pjxt/personnelEvaluation/listObtainPersonnelEvaluationTasks?yhdm={username}&pageNum=1&pageSize=10"
        );
        let res = self.get(url).send().await?.bytes().await?;
        let rwid = match utils::parse_by_tag(&res, "\"rwid\":\"", "\"") {
            Some(rwid) => rwid,
            None => return Err(Error::server("[Tes] Get list failed. No rwid")),
        };

        // 看不懂, 但需要获取一些称为 wjid 的东西, 对应于理论课, 实践课, 英语课, 体育课, 科研课堂, 这是已知的五个类型
        // 省略的无用查询参数 &sfyp=0&pageNum=1&pageSize=999
        let url = format!(
            "https://spoc.buaa.edu.cn/pjxt/evaluationMethodSix/getQuestionnaireListToTask?rwid={rwid}"
        );
        let bytes = self.get(url).send().await?.bytes().await?;

        // 这里需要循环匹配多个 wjid
        let left = "\"wjid\":\"";
        let right = "\"";
        let mut wjids = Vec::new();
        let mut start_index = 0;
        while let Some(s) = utils::parse_by_tag(&bytes[start_index..], left, right) {
            wjids.push(s);
            // 跳过当前匹配的部分以及右标签
            start_index = s.as_ptr() as usize - bytes.as_ptr() as usize + s.len() + right.len();
        }

        // 考虑到一个课可能有很多老师, 需要评多次, 但 30 位应该足够多数人了
        let mut list = Vec::<EvaluationListItem>::with_capacity(30);
        // TODO: 并发请求
        for wjid in wjids {
            // 省略的无用查询参数 &sfyp=0&xnxq=2024-20251&pageNum=1&pageSize=999
            let url = format!(
                "https://spoc.buaa.edu.cn/pjxt/evaluationMethodSix/getRequiredReviewsData?wjid={wjid}"
            );
            let res = self.get(url).send().await?.bytes().await?;
            let res = serde_json::from_slice::<_EvaluationList>(&res)?;
            list.extend(res.list);
        }

        Ok(list)
    }

    pub async fn get_evaluation_form(
        &self,
        item: &EvaluationListItem,
    ) -> crate::Result<EvaluationForm> {
        // 无用查询参数太多了懒得记
        let query = [
            ("rwid", &item.rwid),
            ("wjid", &item.wjid),
            ("sxz", &item.sxz),
            ("pjrdm", &item.pjrdm),
            ("pjrmc", &item.pjrmc),
            ("bpdm", &item.bpdm),
            ("bpmc", &item.teacher),
            ("kcdm", &item.kcdm),
            ("kcmc", &item.course),
            ("rwh", &item.rwh),
        ];
        let url = "https://spoc.buaa.edu.cn/pjxt/evaluationMethodSix/getQuestionnaireTopic";
        let res = self.get(url).query(&query).send().await?.bytes().await?;
        let res = serde_json::from_slice::<_EvaluationForm>(&res)?;
        Ok(res.result)
    }

    pub async fn submit_evaluation(
        &self,
        complete: EvaluationCompleted<'_>,
    ) -> crate::Result<reqwest::Response> {
        let url = "https://spoc.buaa.edu.cn/pjxt/evaluationMethodSix/submitSaveEvaluation";
        let res = self.post(url).json(&complete).send().await?;

        let rwid = complete.rwid();
        let wjid = complete.wjid();
        // 也许是用于验证是否提交成功的
        let query = [("rwid", rwid), ("wjid", wjid)];
        self.post(
            "https://spoc.buaa.edu.cn/pjxt/personnelEvaluation/checkWhetherTheTaskIsEvaluable",
        )
        .query(&query)
        .send()
        .await?;
        self.post("https://spoc.buaa.edu.cn/pjxt/system/property")
            .send()
            .await?;

        // {"code":"200","msg":"成功","msg_en":"Operation is successful","result":[{"pjid":"","pjbm":"","sfnm":"1"}]} 是否匿名??
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[ignore]
    #[tokio::test]
    async fn test_get_evaluation_list() {
        let context = Context::with_auth("./data");

        let tes = context.tes();
        tes.login().await.unwrap();

        let list = tes.get_evaluation_list().await.unwrap();

        let form = tes.get_evaluation_form(&list[0]).await.unwrap();
        println!("{:#?}", form);
    }

    #[ignore]
    #[tokio::test]
    async fn test_submit_evaluation() {
        use crate::api::tes::data::EvaluationAnswer;

        let context = Context::with_auth("./data");

        let tes = context.tes();
        tes.login().await.unwrap();
        let list = tes.get_evaluation_list().await.unwrap();

        for i in list {
            if i.state == true {
                continue;
            }

            let form = tes.get_evaluation_form(&i).await.unwrap();

            let ans = vec![
                EvaluationAnswer::Choice(1),
                EvaluationAnswer::Choice(0),
                EvaluationAnswer::Choice(0),
                EvaluationAnswer::Choice(0),
                EvaluationAnswer::Choice(0),
                EvaluationAnswer::Choice(0),
                EvaluationAnswer::Completion(""),
                EvaluationAnswer::Completion(""),
            ];

            let complete = form.fill(ans);

            let res = tes.submit_evaluation(complete).await.unwrap();

            println!("{}", res.text().await.unwrap());

            // 休眠一秒, 防止请求过快被服务器拒绝
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}
