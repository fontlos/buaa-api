use crate::{Error, utils};

use super::{EvaluationCompleted, EvaluationForm, EvaluationList, EvaluationListItem};

impl super::TesAPI {
    /// Teacher Evaluation System Login
    pub async fn login(&self) -> crate::Result<()> {
        // 登录
        let login_url =
            "https://sso.buaa.edu.cn/login?service=https%3A%2F%2Fspoc.buaa.edu.cn%2Fpjxt%2Fcas";
        let res = self.get(login_url).send().await?;
        if res.url().as_str() == login_url {
            return Err(Error::LoginExpired("SSO Expired".to_string()));
        }
        Ok(())
    }

    /// Get a list of the ones that need to beevaluated <br>
    /// The method has made multiple requests inside it, and the speed is slow
    pub async fn get_evaluation_list(&self) -> crate::Result<Vec<EvaluationListItem>> {
        // 考虑到 rwid 只有这一个地方用到, 所以直接在这里获取
        // 获取账号
        let config = self.config.load();
        let username = config.username.as_ref().unwrap();
        // 获取 rwid
        // 省略的无用查询参数 &rwmc=&sfyp=0
        let url = format!(
            "https://spoc.buaa.edu.cn/pjxt/personnelEvaluation/listObtainPersonnelEvaluationTasks?yhdm={username}&pageNum=1&pageSize=10"
        );
        let res = self.get(url).send().await?;
        let text = res.text().await?;
        let rwid = match utils::get_value_by_lable(&text, r#""rwid":""#, "\"") {
            Some(rwid) => rwid,
            None => return Err(Error::APIError("No rwid".to_string())),
        };

        // 看不懂, 但需要获取一些称为 wjid 的东西, 对应于理论课, 实践课, 英语课, 体育课, 科研课堂, 这是已知的五个类型
        // 省略的无用查询参数 &sfyp=0&pageNum=1&pageSize=999
        let url = format!(
            "https://spoc.buaa.edu.cn/pjxt/evaluationMethodSix/getQuestionnaireListToTask?rwid={rwid}"
        );
        let list = self.get(url).send().await?;
        let text = list.text().await?;
        let wjids = utils::get_values_by_lable(&text, r#""wjid":""#, "\"");

        let mut list = Vec::<EvaluationListItem>::with_capacity(20);
        for wjid in wjids {
            // 省略的无用查询参数 &sfyp=0&xnxq=2024-20251&pageNum=1&pageSize=999
            let url = format!(
                "https://spoc.buaa.edu.cn/pjxt/evaluationMethodSix/getRequiredReviewsData?wjid={wjid}"
            );
            let res = self.get(url).send().await?;
            let text = res.text().await?;
            let new_list: EvaluationList = serde_json::from_str(&text)?;
            list.extend(new_list.list);
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
        let res = self.get(url).query(&query).send().await?;
        let text = res.text().await?;
        let form: EvaluationForm = serde_json::from_str(&text)?;
        Ok(form)
    }

    pub async fn submit_evaluation(
        &self,
        complete: EvaluationCompleted,
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
        .await
        .unwrap();
        self.post("https://spoc.buaa.edu.cn/pjxt/system/property")
            .send()
            .await
            .unwrap();

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::utils::env;

    #[ignore]
    #[tokio::test]
    async fn test_get_evaluation_list() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);
        context.login().await.unwrap();

        let tes = context.tes();
        tes.login().await.unwrap();
        let list = tes.get_evaluation_list().await.unwrap();
        println!("{:?}", list);
    }

    #[ignore]
    #[tokio::test]
    async fn test_submit_evaluation() {
        use crate::api::tes::utils::EvaluationAnswer;

        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);
        context.login().await.unwrap();

        let tes = context.tes();
        tes.login().await.unwrap();
        let list = tes.get_evaluation_list().await.unwrap();

        let last = list.get(2).unwrap();

        let form = tes.get_evaluation_form(last).await.unwrap();

        let ans = vec![
            EvaluationAnswer::Choice(1),
            EvaluationAnswer::Choice(0),
            EvaluationAnswer::Choice(0),
            EvaluationAnswer::Choice(0),
            EvaluationAnswer::Choice(0),
            EvaluationAnswer::Choice(0),
            EvaluationAnswer::Completion("".to_string()),
            EvaluationAnswer::Completion("".to_string()),
        ];

        let complete = form.fill(ans);

        let res = tes.submit_evaluation(complete).await.unwrap();

        println!("{}", res.text().await.unwrap());
    }
}
