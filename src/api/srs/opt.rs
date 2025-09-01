use crate::api::Location;
use crate::error::Error;

use super::{_SrsRes1, _SrsRes2, _SrsStatus, SrsCourses, SrsFilter, SrsOpt, SrsSelected};

impl super::SrsApi {
    /// Get the default course filter.
    /// If you know your campus,
    /// you can create the filter by `SrsFilter::new` directly.
    pub async fn get_default_filter(&self) -> crate::Result<SrsFilter> {
        let url = "https://byxk.buaa.edu.cn/xsxk/web/studentInfo";

        // 获取 token
        let cred = self.cred.load();
        let token = match cred.srs_token.value() {
            Some(t) => t,
            None => return Err(Error::auth_expired(Location::Srs)),
        };

        let query = [("token", token)];

        let res = self
            .post(url)
            .header("Authorization", token)
            .query(&query)
            .send()
            .await?;
        let text = res.text().await?;
        let campus = crate::utils::get_value_by_lable(&text, "\"campus\": \"", "\"")
            .and_then(|campus| campus.parse::<u8>().ok())
            .map(SrsFilter::new)
            .ok_or_else(|| Error::server("[Srs] Invalid or missing campus"))?;
        Ok(campus)
    }

    /// Query Course
    pub async fn query_course(&self, filter: &SrsFilter) -> crate::Result<SrsCourses> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/buaa/clazz/list";

        // 获取 token
        let cred = self.cred.load();
        let token = match cred.srs_token.value() {
            Some(t) => t,
            None => return Err(Error::auth_expired(Location::Srs)),
        };

        let res = self
            .post(url)
            .json(&filter)
            .header("Authorization", token)
            .send()
            .await?;
        let text = res.text().await?;
        let status = serde_json::from_str::<_SrsStatus>(&text)?;
        if status.code != 200 {
            return Err(Error::server(format!("[Srs] Response: {}", status.msg)));
        }

        let res = serde_json::from_str::<_SrsRes1>(&text)?;
        Ok(res.data)
    }

    /// Query Selected Course
    pub async fn query_selected(&self) -> crate::Result<Vec<SrsSelected>> {
        // 查询退选记录的 URL, 操作相同, 但感觉没啥用
        // https://byxk.buaa.edu.cn/xsxk/elective/deselect
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/select";

        // 获取 token
        let cred = self.cred.load();
        let token = match cred.srs_token.value() {
            Some(t) => t,
            None => return Err(Error::auth_expired(Location::Srs)),
        };

        let res = self.post(url).header("Authorization", token).send().await?;
        let text = res.text().await?;
        let status = serde_json::from_str::<_SrsStatus>(&text)?;
        if status.code != 200 {
            return Err(Error::server(format!("[Srs] Response: {}", status.msg)));
        }
        let res = serde_json::from_str::<_SrsRes2>(&text)?;
        Ok(res.data)
    }

    /// # Select Course
    /// Note that you cannot call the login to update the token before calling this function, otherwise the verification will fail
    pub async fn select_course<'a>(&self, opt: &'a SrsOpt<'a>) -> crate::Result<()> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/buaa/clazz/add";

        // 获取 token
        let cred = self.cred.load();
        let token = match cred.srs_token.value() {
            Some(t) => t,
            None => return Err(Error::auth_expired(Location::Srs)),
        };

        let res = self
            .post(url)
            .form(&opt)
            .header("Authorization", token)
            .send()
            .await?
            .json::<_SrsStatus>()
            .await?;
        if res.code != 200 {
            return Err(Error::server(format!("[Srs] Response: {}", res.msg)));
        }
        Ok(())
    }

    /// # Drop Course
    /// Note that you cannot call the login to update the token before calling this function, otherwise the verification will fail
    pub async fn drop_course<'a>(&self, opt: &'a SrsOpt<'a>) -> crate::Result<()> {
        let url = "https://byxk.buaa.edu.cn/xsxk/elective/clazz/del";

        // 获取 token
        let cred = self.cred.load();
        let token = match cred.srs_token.value() {
            Some(t) => t,
            None => return Err(Error::auth_expired(Location::Srs)),
        };

        let res = self
            .post(url)
            .form(&opt)
            .header("Authorization", token)
            .send()
            .await?
            .json::<_SrsStatus>()
            .await?;
        if res.code != 200 {
            return Err(Error::server(format!("[Srs] Response: {}", res.msg)));
        }
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
        srs.login().await.unwrap();

        let filter = srs.get_default_filter().await.unwrap();

        let res = srs.query_course(&filter).await.unwrap();
        println!("{:?}", res);
    }

    #[ignore]
    #[tokio::test]
    async fn test_srs_query_selete() {
        let context = Context::with_auth("./data");

        let srs = context.srs();
        srs.login().await.unwrap();

        let res = srs.query_selected().await.unwrap();

        println!("{:?}", res);
    }
}
