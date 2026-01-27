#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[ignore]
    #[tokio::test]
    async fn test_spoc() {
        let context = Context::with_auth("./data").unwrap();

        let spoc = context.spoc();

        let res = spoc.get_week().await.unwrap();
        println!("{:?}", res);
        let res = spoc.query_week_schedules(&res).await.unwrap();
        println!("{:?}", res);
        context.save_auth("./data").unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_spoc_upload() {
        let context = Context::with_auth("./data").unwrap();

        let spoc = context.spoc();

        let file = std::fs::File::open("data/file.pdf").unwrap();
        let upload = spoc.upload(file, "file.pdf").await.unwrap();
        println!("URL: {}", upload.as_url());

        // let courses = spoc.query_courses("2025-20261").await.unwrap();
        // let course = &courses[1];
        // let homeworks = spoc.query_homeworks(course).await.unwrap();
        // let homework = &homeworks[12];
        // let info = spoc.query_homework_detail(homework).await.unwrap();
        // println!("{:#?}", info);

        // spoc.submit_homework(homework, &upload).await.unwrap();

        context.save_auth("./data").unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_spoc_upload_with_progress() {
        use buaa_api::api::spoc::UploadArgs;

        let file_path = "data/file.zip";

        let mut file = std::fs::File::open(file_path).unwrap();
        let args = tokio::task::spawn_blocking(move || {
            UploadArgs::from_reader(&mut file, "file.zip".into()).unwrap()
        })
        .await
        .unwrap();

        let context = Context::with_auth("./data").unwrap();
        let spoc = context.spoc();

        let res = spoc.upload_fast(&args).await.unwrap();
        if let Some(res) = res {
            println!("Fast upload hit cache, URL: {}", res.as_url());
        } else {
            let data = std::fs::File::open(file_path).unwrap();
            let mut handle = spoc.upload_progress(args, data);

            tokio::spawn(async move {
                while let Some(progress) = handle.progress_rx.recv().await {
                    println!("Progress: {}/{}", progress.done, progress.total);
                }
            });

            match handle.result_rx.await {
                Ok(Ok(res)) => println!("URL: {}", res.as_url()),
                Ok(Err(e)) => println!("Upload Error: {}", e),
                Err(_) => println!("Channel Err"),
            }
        }

        context.save_auth("./data").unwrap();
    }
}

fn main() {}
