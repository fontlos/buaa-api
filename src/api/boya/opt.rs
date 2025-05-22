use super::utils::{_BoyaAttend, BoyaAttend, BoyaCoordinate};

impl super::BoyaAPI {
    /// # Select Course
    /// - Input: Course ID from [`query_course`](#method.query_course)
    /// - Output: Status of the request, like `{"status":"0","errmsg":"请求成功","token":null,"data":{"courseCurrentCount":340}}`
    pub async fn select_course(&self, id: u32) -> crate::Result<String> {
        let query = format!("{{\"courseId\":{id}}}");
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/choseCourse
        let url = "https://bykc.buaa.edu.cn/sscv/choseCourse";
        let res = self.universal_request(&query, url).await?;
        Ok(res)
    }

    /// # Drop Course
    /// - Input: Course ID from [`query_course`](#method.query_course)
    /// - Output: Status of the request, like `{"status":"0","errmsg":"请求成功","token":null,"data":{"courseCurrentCount":340}}`
    pub async fn drop_course(&self, id: u32) -> crate::Result<String> {
        let query = format!("{{\"id\":{id}}}");
        // TODO: VPN 方法使用下面的 URL, 但我还没想好怎么分组
        // https://d.buaa.edu.cn/https/77726476706e69737468656265737421f2ee4a9f69327d517f468ca88d1b203b/sscv/delChosenCourse
        let url = "https://bykc.buaa.edu.cn/sscv/delChosenCourse";
        let res = self.universal_request(&query, url).await?;
        Ok(res)
    }

    // 这个接口只在 Android UA 时才能找到, 但不妨碍使用, 在浏览器调试时可以尝试修改 UA
    // TODO: 也许我可以考虑全局使用 Android UA 避免一些痕迹
    pub async fn attend_course(
        &self,
        id: u32,
        coordinate: BoyaCoordinate,
    ) -> crate::Result<BoyaAttend> {
        use rand::Rng;
        let mut rng = rand::rng();
        let offset = 1e-5;

        let lng_offset = rng.random_range(-offset..offset);
        let lat_offset = rng.random_range(-offset..offset);

        let query = format!(
            "{{\"courseId\":{},\"signLat\":{},\"signLng\":{},\"signType\":2}}",
            id,
            coordinate.latitude + lat_offset,
            coordinate.longitude + lng_offset
        );
        let url = "https://bykc.buaa.edu.cn/sscv/signCourseByUser";
        let res = self.universal_request(&query, url).await?;
        let res = serde_json::from_str::<_BoyaAttend>(&res)?;
        Ok(res.data.info)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::utils::env;

    #[ignore]
    #[tokio::test]
    async fn test_boya_select() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);
        context.login().await.unwrap();

        let boya = context.boya();
        boya.login().await.unwrap();
        let res = boya.select_course(6637).await.unwrap();
        println!("{}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_drop() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);
        context.login().await.unwrap();

        let boya = context.boya();
        boya.login().await.unwrap();
        let res = boya.drop_course(6637).await.unwrap();
        println!("{}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_boya_checkin() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);

        let boya = context.boya();
        let id = 7882;

        let coordinate = boya
            .query_attend_rule(id)
            .await
            .unwrap()
            .unwrap()
            .coordinate;
        let res = boya.attend_course(id, coordinate).await.unwrap();
        println!("{:?}", res);
    }
}
