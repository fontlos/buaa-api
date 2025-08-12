impl super::AppApi {
    pub async fn classtable_get_index(&self) -> crate::Result<reqwest::Response> {
        let res = self
            .get("https://app.buaa.edu.cn/timetable/wap/default/get-index")
            .send()
            .await?;
        Ok(res)
    }

    pub async fn classtable_get_data(&self) -> crate::Result<reqwest::Response> {
        let form = [
            ("year", "2024-2025"),
            ("term", "1"),
            ("week", "13"),
            ("type", "2"),
        ];
        let res = self
            .post("https://app.buaa.edu.cn/timetable/wap/default/get-datatmp")
            .form(&form)
            .send()
            .await?;
        Ok(res)
    }
}
