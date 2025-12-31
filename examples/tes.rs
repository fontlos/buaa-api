#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_tes() {
        let context = Context::with_auth("./data").unwrap();

        let tes = context.tes();

        let tasks = tes.get_task().await.unwrap_or(Vec::new());

        for t in tasks {
            if t.state == true {
                continue;
            }
            let form = tes.get_form(&t).await.unwrap();
            let complete = form.fill_default();
            tes.submit_form(complete).await.unwrap();
            println!("'{}' success", t.course);
            // 休眠一秒, 防止请求过快被服务器拒绝
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        context.save_auth("./data").unwrap();
    }
}

fn main() {}
