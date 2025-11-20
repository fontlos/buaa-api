#[cfg(test)]
mod tests {
    use buaa_api::Context;

    #[tokio::test]
    async fn test_dir() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();

        let user_dir = cloud.get_user_dir_id().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();
        println!("List: {list:#?}");
        let size = cloud.get_item_size(&user_dir).await.unwrap();
        println!("Size: {size:#?}");

        context.save_auth("./data");
    }

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

    #[tokio::test]
    async fn test_rename() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir_id().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();

        cloud.rename_item(&list.dirs[0].id, "dir").await.unwrap();

        context.save_auth("./data");
    }

    #[tokio::test]
    async fn test_move() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir_id().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();
        let dir = &list.dirs[0];
        let file = &list.files[0];

        let res = cloud.move_item(&dir.id, &file.id).await.unwrap();

        println!("Moved item id: {}", res);

        context.save_auth("./data");
    }

    #[tokio::test]
    async fn test_create() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir_id().await.unwrap();
        let name = cloud.get_suggest_name(&user_dir).await.unwrap();
        let _ = cloud.create_dir(&user_dir, &name).await.unwrap();

        context.save_auth("./data");
    }

    #[tokio::test]
    async fn test_fast_upload() {
        use buaa_api::crypto;
        let file = std::fs::read("./data/c.bat").unwrap();
        let length = file.len() as u64;
        let md5 = crypto::bytes2hex(&crypto::md5::Md5::digest(&file));
        let crc32 = format!("{:08x}", crypto::crc::Crc32::digest(&file));

        let context = Context::with_auth("./data");

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir_id().await.unwrap();

        let res = cloud.check_hash(&md5, length).await.unwrap();
        println!("Hash exists: {}", res);
        if res {
            let res = cloud
                .fast_upload(&user_dir, "c.bat", length, &md5, &crc32)
                .await
                .unwrap();
            println!("Fast upload success: {}", res);
        }

        context.save_auth("./data");
    }

    #[tokio::test]
    async fn test_upload() {
        let file = std::fs::read("./data/c.bat").unwrap();
        let length = file.len() as u64;

        let context = Context::with_auth("./data");

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir_id().await.unwrap();

        let filename = "c.bat";
        let res = cloud
            .upload_auth(&user_dir, filename, length)
            .await
            .unwrap();

        println!("Upload Args: {:?}", &res);

        #[cfg(feature = "multipart")]
        {
            let part = buaa_api::exports::Part::bytes(file);
            cloud.upload(res, part).await.unwrap();
        }

        context.save_auth("./data");
    }

    #[tokio::test]
    async fn test_share() {
        let context = Context::with_auth("./data");

        let cloud = context.cloud();
        let user_dir = cloud.get_user_dir_id().await.unwrap();
        let list = cloud.list_dir(&user_dir).await.unwrap();

        let share = list.dirs[0].to_share();
        let share_id = cloud.share_item(&share).await.unwrap();
        println!("Share Link: https://bhpan.buaa.edu.cn/link/{}", share_id);

        let share = share.enable_preview().enable_upload();
        cloud.share_update(&share_id, &share).await.unwrap();
        cloud.share_delete(&share_id).await.unwrap();

        context.save_auth("./data");
    }
}

fn main() {}
