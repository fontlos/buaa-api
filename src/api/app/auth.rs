impl super::AppAPI {
    pub async fn login(&self) -> crate::Result<()> {
        self.get("https://app.buaa.edu.cn/uc/wap/login")
            .send()
            .await?;
        Ok(())
    }
}
