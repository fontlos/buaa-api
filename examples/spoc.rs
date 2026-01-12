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

        let file = || std::fs::File::open("data/file.pdf").unwrap();
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
}

fn main() {}
