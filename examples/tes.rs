#[cfg(test)]
mod tests {
    use buaa_api::Context;

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

fn main() {}