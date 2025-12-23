use crate::{Error, utils};

use super::{Course, Data, Filter, Opt, Payload, Selected};

impl super::SrsApi {
    /// # Get the default course filter.
    ///
    /// If you know your campus,
    /// you can create the filter by [Filter::new()] directly.
    pub async fn get_default_filter(&self) -> crate::Result<Filter> {
        let url = "https://byxk.buaa.edu.cn/xsxk/web/studentInfo";
        let payload = Payload::<'_, ()>::QueryWithToken;
        let res: serde_json::Value = self.universal_request(url, payload).await?;
        let campus = res
            .pointer("/student/campus")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::server("Missing campus field").with_label("Srs"))?
            .parse::<u8>()
            .map_err(|_| Error::parse("Failed to parse campus"))?;
        Ok(Filter::new(campus))
    }

    // 预选所需, 有病吧嵌在 HTML 里
    /// # Get the batch id for course selection.
    ///
    /// **Note**: Do not need login
    ///
    /// **Note**: Only for [super::SrsApi::pre_select_course]
    pub async fn get_batch(&self) -> crate::Result<String> {
        let url = "https://byxk.buaa.edu.cn/xsxk/profile/index.html";
        let bytes = self.client.get(url).send().await?.bytes().await?;
        let id = utils::parse_by_tag(&bytes, "\"code\":\"", "\"");
        match id {
            Some(i) => Ok(i.to_string()),
            None => Err(Error::server("Cannot find batch id").with_label("Srs")),
        }
    }

    /// # Query Course
    pub async fn query_course(&self, filter: &Filter) -> crate::Result<Vec<Course>> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/buaa/clazz/list";
        let payload = Payload::Json(filter);
        let mut res: Data<Vec<Course>> = self.universal_request(url, payload).await?;
        // 手动插入 scope, 方便调用
        res.0.iter_mut().for_each(|c| c.scope = filter.scope);
        Ok(res.0)
    }

    /// # Query Pre-Selected Course
    ///
    /// **Note**: Only for pre-selection. Late-selection use `query_selected`
    ///
    /// **Note**: Collect into `Vec` according to volunteer grouping
    pub async fn query_pre_selected(&self) -> crate::Result<Vec<Vec<Selected>>> {
        let url = "https://byxk.buaa.edu.cn/xsxk/volunteer/select";
        let payload = Payload::<'_, ()>::Empty;
        let res: Data<Vec<Vec<Selected>>> = self.universal_request(url, payload).await?;
        Ok(res.0)
    }

    /// # Query Selected Course
    ///
    /// **Note**: Only for late-selection. Pre-selection use `query_pre_selected`
    pub async fn query_selected(&self) -> crate::Result<Vec<Selected>> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/select";
        let payload = Payload::<'_, ()>::Empty;
        let res = self.universal_request(url, payload).await?;
        Ok(res)
    }

    // 查询退选记录的 URL, 操作与上面相同, 但感觉没啥用
    // https://byxk.buaa.edu.cn/xsxk/elective/deselect

    // 什么**玩意, 课程列表缺少 range 参数, opt 还得从过滤器里借一个,
    // 还比退选多了个莫名其妙的 batch 参数, 这玩意还写死在 HTML 里
    /// # Pre-Select Course
    ///
    /// **Note**: Only for pre-selection. Late-selection use `select_course`
    ///
    /// **Note**: You cannot call login before calling this, otherwise the verification will fail
    ///
    /// - Input: `opt`: call `as_opt` on [Course]. And must call `set_batch` and `set_index` on [Opt]
    pub async fn pre_select_course<'a>(&self, opt: &'a Opt<'a>) -> crate::Result<()> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/buaa/clazz/add";
        let payload = Payload::Form(&opt);
        let _: Option<()> = self.universal_request(url, payload).await?;

        Ok(())
    }

    /// # Select Course
    ///
    /// **Note**: Only for late-selection. Pre-selection use `pre_select_course`
    ///
    /// **Note**: You cannot call login before calling this, otherwise the verification will fail
    ///
    /// - Input: `opt`: call `as_opt` on [Course]
    pub async fn select_course<'a>(&self, opt: &'a Opt<'a>) -> crate::Result<()> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/buaa/clazz/add";
        let payload = Payload::Form(opt);
        let _: Option<()> = self.universal_request(url, payload).await?;

        Ok(())
    }

    /// # Drop Course
    ///
    /// **Note**: You cannot call login before calling this, otherwise the verification will fail
    ///
    /// - Input: `opt`: call `as_opt` on [Course] or [Selected]
    pub async fn drop_course<'a>(&self, opt: &'a Opt<'a>) -> crate::Result<()> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/clazz/del";
        let payload = Payload::Form(opt);
        let _: Option<()> = self.universal_request(url, payload).await?;

        Ok(())
    }
}
