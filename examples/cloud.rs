#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[tokio::test]
    async fn test_get_url() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();

        let user_dir = cloud.get_user_dir_id().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();
        let url = cloud.get_download_url(&list.files, &[0]).await.unwrap();

        println!("Download URL: {url}");

        context.save_auth("./data");
    }

    #[tokio::test]
    async fn test_delete() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir_id().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();

        cloud.delete_item(&list.files[0].id).await.unwrap();

        context.save_auth("./data");
    }
}

fn main() {}