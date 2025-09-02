use crate::{Error, utils};

use super::{_SrsBody, _SrsCampus, SrsCourses, SrsFilter, SrsOpt, SrsSelected};

impl super::SrsApi {
    /// Get the default course filter.
    /// If you know your campus,
    /// you can create the filter by `SrsFilter::new` directly.
    pub async fn get_default_filter(&self) -> crate::Result<SrsFilter> {
        let url = "https://byxk.buaa.edu.cn/xsxk/web/studentInfo";
        let body = _SrsBody::<'_, ()>::QueryToken;
        let res: _SrsCampus = self.universal_request(url, body).await?;
        Ok(SrsFilter::new(res.0))
    }

    // 预选所需, 有病吧嵌在 HTML 里
    /// **Note**: Do not need login
    pub async fn get_batch_id(&self) -> crate::Result<String> {
        let url = "https://byxk.buaa.edu.cn/xsxk/profile/index.html";
        let res = self.client.get(url).send().await?;
        let text = res.text().await?;
        let id = utils::get_value_by_lable(&text, "\"code\":\"", "\"");
        match id {
            Some(i) => Ok(i.to_string()),
            None => Err(Error::server("Cannot find batch id")),
        }
    }

    /// Query Course
    pub async fn query_course(&self, filter: &SrsFilter) -> crate::Result<SrsCourses> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/buaa/clazz/list";
        let body = _SrsBody::Json(filter);
        let res = self.universal_request(url, body).await?;
        Ok(res)
    }

    /// Query Selected Course
    pub async fn query_selected(&self) -> crate::Result<Vec<SrsSelected>> {
        // 查询退选记录的 URL, 操作相同, 但感觉没啥用
        // https://byxk.buaa.edu.cn/xsxk/elective/deselect

        // 用于补选中查询已选, 预选中查询用
        // https://byxk.buaa.edu.cn/xsxk/volunteer/select
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/select";
        let body = _SrsBody::<'_, ()>::None;
        let res = self.universal_request(url, body).await?;
        Ok(res)
    }

    /// # Select Course
    /// Note that you cannot call the login to update the token before calling this function, otherwise the verification will fail
    pub async fn select_course<'a>(&self, opt: &'a SrsOpt<'a>) -> crate::Result<()> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/buaa/clazz/add";
        let body = _SrsBody::Form(opt);
        let _: Option<()> = self.universal_request(url, body).await?;

        Ok(())
    }

    /// # Drop Course
    /// Note that you cannot call the login to update the token before calling this function, otherwise the verification will fail
    pub async fn drop_course<'a>(&self, opt: &'a SrsOpt<'a>) -> crate::Result<()> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/clazz/del";
        let body = _SrsBody::Form(opt);
        let _: Option<()> = self.universal_request(url, body).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[ignore]
    #[tokio::test]
    async fn test_srs_query() {
        let context = Context::with_auth("./data");

        let srs = context.srs();

        let filter = srs.get_default_filter().await.unwrap();

        let res = srs.query_course(&filter).await.unwrap();
        println!("{:?}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_srs_query_selete() {
        let context = Context::with_auth("./data");

        let srs = context.srs();

        let res = srs.query_selected().await.unwrap();

        println!("{:?}", res);
    }
}
