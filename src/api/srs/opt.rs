use crate::{Error, utils};

use super::{_PreSelecteds, Body, Course, Courses, Filter, Opt, Selected};

impl super::SrsApi {
    /// Get the default course filter.
    /// If you know your campus,
    /// you can create the filter by [Filter::new()] directly.
    pub async fn get_default_filter(&self) -> crate::Result<Filter> {
        let url = "https://byxk.buaa.edu.cn/xsxk/web/studentInfo";
        let body = Body::<'_, ()>::QueryToken;
        let res: serde_json::Value = self.universal_request(url, body).await?;
        let campus = res
            .pointer("/student/campus")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::server("Missing campus field").with_label("Srs"))?
            .parse::<u8>()
            .map_err(|_| Error::parse("Failed to parse campus"))?;
        Ok(Filter::new(campus))
    }

    // 预选所需, 有病吧嵌在 HTML 里
    /// Get the batch id for course selection.
    ///
    /// **Note**: Do not need login
    pub async fn get_batch(&self) -> crate::Result<String> {
        let url = "https://byxk.buaa.edu.cn/xsxk/profile/index.html";
        let res = self.client.get(url).send().await?.bytes().await?;
        let id = utils::parse_by_tag(&res, "\"code\":\"", "\"");
        match id {
            Some(i) => Ok(i.to_string()),
            None => Err(Error::server("Cannot find batch id").with_label("Srs")),
        }
    }

    /// Query Course
    pub async fn query_course(&self, filter: &Filter) -> crate::Result<Courses> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/buaa/clazz/list";
        let body = Body::Json(filter);
        let res = self.universal_request(url, body).await?;
        Ok(res)
    }

    /// Query Pre-Selected Course
    pub async fn query_pre_selected(&self) -> crate::Result<Vec<Selected>> {
        // 查询退选记录的 URL, 操作相同, 但感觉没啥用
        // https://byxk.buaa.edu.cn/xsxk/elective/deselect

        let url = "https://byxk.buaa.edu.cn/xsxk/volunteer/select";
        let body = Body::<'_, ()>::None;
        let res: Vec<_PreSelecteds> = self.universal_request(url, body).await?;
        let res = res.into_iter().flat_map(|g| g.list).collect();
        Ok(res)
    }

    /// Query Selected Course
    pub async fn query_selected(&self) -> crate::Result<Vec<Selected>> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/select";
        let body = Body::<'_, ()>::None;
        let res = self.universal_request(url, body).await?;
        Ok(res)
    }

    /// # Pre-Select Course
    ///
    /// **Note**: Only for pre-selection. Late-selection use `select_course`
    ///
    /// **Note**: You cannot call login before calling this, otherwise the verification will fail
    ///
    /// - Input
    ///     - `c`: Course to select, obtained from `query_course`
    ///     - `f`: Filter current used
    ///     - `b`: Batch ID, obtained from `get_batch`
    ///     - `i`: Index ID
    pub async fn pre_select_course(
        &self,
        c: &Course,
        f: &Filter,
        b: &str,
        i: u8,
    ) -> crate::Result<()> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/buaa/clazz/add";
        // 什么 ** 玩意, 课程列表缺少 range 参数还得从过滤器里借一个,
        // 还比退选多了个莫名其妙的 batch 参数, 这玩意还写死在 HTML 里
        let form = [
            ("clazzType", f.scope.as_str()),
            ("clazzId", &c.id),
            ("secretVal", &c.sum),
            ("batchId", b),
            ("chooseVolunteer", &i.to_string()),
        ];
        let body = Body::Form(&form);
        let _: Option<()> = self.universal_request(url, body).await?;

        Ok(())
    }

    /// # Select Course
    ///
    /// **Note**: Only for late-selection. Pre-selection use `pre_select_course`
    ///
    /// **Note**: You cannot call login before calling this, otherwise the verification will fail
    ///
    /// - Input: `opt`: call `as_opt` on `SrsCourse` or `SrsSelected`
    pub async fn select_course<'a>(&self, opt: &'a Opt<'a>) -> crate::Result<()> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/buaa/clazz/add";
        let body = Body::Form(opt);
        let _: Option<()> = self.universal_request(url, body).await?;

        Ok(())
    }

    /// # Drop Course
    ///
    /// **Note**: You cannot call login before calling this, otherwise the verification will fail
    ///
    /// - Input: `opt`: call `as_opt` on `SrsCourse` or `SrsSelected`
    pub async fn drop_course<'a>(&self, opt: &'a Opt<'a>) -> crate::Result<()> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/clazz/del";
        let body = Body::Form(opt);
        let _: Option<()> = self.universal_request(url, body).await?;

        Ok(())
    }
}
