#[cfg(test)]
mod tests {
    use buaa_api::Context;

    fn init_log() {
        use logforth::append;
        use logforth::layout::TextLayout;
        use logforth::record::Level;
        use logforth::record::LevelFilter;

        logforth::starter_log::builder()
            .dispatch(|d| {
                d.filter(LevelFilter::MoreSevereEqual(Level::Info))
                    .append(append::Stdout::default().with_layout(TextLayout::default()))
            })
            .apply();
    }

    #[tokio::test]
    async fn test_dir() {
        init_log();

        let context = Context::with_auth("./data").unwrap();

        let cloud = context.cloud();

        let user_dir = cloud.get_user_dir().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();
        println!("List: {list:#?}");
        let size = cloud.get_item_size(&user_dir).await.unwrap();
        println!("Size: {size:#?}");

        context.save_auth("./data").unwrap();
    }

    #[tokio::test]
    async fn test_create() {
        let context = Context::with_auth("./data").unwrap();

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir().await.unwrap();
        let name = cloud
            .get_suggest_name(&user_dir, "New Folder")
            .await
            .unwrap();
        let _ = cloud.create_dir(&user_dir, &name).await.unwrap();

        context.save_auth("./data").unwrap();
    }

    #[tokio::test]
    async fn test_rename() {
        init_log();

        let context = Context::with_auth("./data").unwrap();

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();

        cloud.rename_item(&list.files[0], "dir").await.unwrap();

        context.save_auth("./data").unwrap();
    }

    #[tokio::test]
    async fn test_move() {
        init_log();

        let context = Context::with_auth("./data").unwrap();

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();
        let item1 = &list.dirs[0];
        let item2 = &list.dirs[1];

        let res = cloud.move_item(&item2, &item1).await.unwrap();

        println!("Moved item id: {}", res);

        context.save_auth("./data").unwrap();
    }

    #[tokio::test]
    async fn test_copy() {
        let context = Context::with_auth("./data").unwrap();

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();
        let dir = &list.dirs[0];
        let file = &list.files[0];

        let res = cloud.copy_item(&file, &dir).await.unwrap();
        println!("Copied item id: {}", res);

        context.save_auth("./data").unwrap();
    }

    #[tokio::test]
    async fn test_delete() {
        let context = Context::with_auth("./data").unwrap();

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();

        cloud.delete_item(&list.dirs[2]).await.unwrap();

        context.save_auth("./data").unwrap();
    }

    #[tokio::test]
    async fn test_recycle() {
        let context = Context::with_auth("./data").unwrap();

        let cloud = context.cloud();

        let recycle = cloud.list_recycle().await.unwrap();
        // println!("Recycle: {recycle:#?}");
        // cloud.delete_recycle_item(&recycle.dirs[0].id).await.unwrap();
        let id = cloud.restore_recycle_item(&recycle.files[0]).await.unwrap();
        println!("Restored: {}", id);

        context.save_auth("./data").unwrap();
    }

    #[tokio::test]
    async fn test_share() {
        init_log();

        let context = Context::with_auth("./data").unwrap();

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();

        let shares = cloud.share_record(&list.files[0]).await.unwrap();
        println!("Shares: {shares:#?}");

        let share = list.files[0].to_share();
        let share = cloud.share_item(share).await.unwrap();
        println!("Share Link: {}", share.as_url());

        let share = share.enable_preview().enable_download();
        cloud.share_update(&share).await.unwrap();
        cloud.share_delete(&share).await.unwrap();

        context.save_auth("./data").unwrap();
    }

    #[tokio::test]
    async fn test_get_download_url() {
        init_log();

        let context = Context::with_auth("./data").unwrap();

        let cloud = context.cloud();

        let user_dir = cloud.get_user_dir().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();
        let url = cloud.get_download_url(&list.files, &[0]).await.unwrap();

        println!("Download URL: {url}");

        context.save_auth("./data").unwrap();
    }

    #[tokio::test]
    async fn test_upload_fast() {
        use buaa_api::api::cloud::UploadArgs;
        use std::fs::File;

        init_log();

        let context = Context::with_auth("./data").unwrap();

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir().await.unwrap();

        let mut reader = File::open("./data/file.zip").unwrap();
        let mut args = UploadArgs::new(&user_dir, "file.zip");
        args.compute_mini(&mut reader).unwrap();

        let res = cloud.upload_fast_check(&args).await.unwrap();
        println!("Can upload fast: {}", res);
        if res {
            args.compute_full(&mut reader).unwrap();
            cloud.upload_fast(&args).await.unwrap();
        }

        context.save_auth("./data").unwrap();
    }

    #[tokio::test]
    async fn test_upload_small() {
        use buaa_api::api::cloud::UploadArgs;
        use std::fs::File;
        use std::io::Read;

        init_log();

        let context = Context::with_auth("./data").unwrap();

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir().await.unwrap();

        let mut args = UploadArgs::new(&user_dir, "file.zip");
        let mut reader = File::open("./data/file.zip").unwrap();
        args.compute_mini(&mut reader).unwrap();

        println!("Upload Args: {args:#?}");

        let mut body = Vec::new();
        reader.read_to_end(&mut body).unwrap();

        cloud.upload_small(&args, body).await.unwrap();

        context.save_auth("./data").unwrap();
    }

    #[tokio::test]
    async fn test_upload_big() {
        use buaa_api::api::cloud::UploadArgs;
        use std::fs::File;

        init_log();

        let context = Context::with_auth("./data").unwrap();

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir().await.unwrap();

        let mut args = UploadArgs::new(&user_dir, "file.zip");
        let mut reader = File::open("./data/file.zip").unwrap();
        args.compute_mini(&mut reader).unwrap();

        println!("Upload Args: {args:#?}");

        cloud.upload_big(&args, reader).await.unwrap();

        context.save_auth("./data").unwrap();
    }
}

fn main() {}
