use serde::{Deserialize, Deserializer};
use serde_json::Value;
use time::{format_description, PrimitiveDateTime, Weekday};

use super::SpocAPI;

#[derive(Deserialize)]
struct SpocRes1 {
    content: SpocWeek,
}

#[derive(Debug, Deserialize)]
pub struct SpocWeek {
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(rename = "pjmrrq")]
    time: (String, String),
    #[serde(rename = "mrxq")]
    pub term: String,
}

pub(super) fn deserialize_time<'de, D>(deserializer: D) -> Result<(String, String), D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let mut s = s.split(",");
    s.next();
    Ok((s.next().unwrap().to_string(), s.next().unwrap().to_string()))
}

#[derive(Deserialize)]
struct SpocRes2 {
    content: Vec<SpocSchedule>,
}

#[derive(Debug, Deserialize)]
pub struct SpocSchedule {
    #[serde(deserialize_with = "deserialize_spoc_day")]
    pub weekday: Weekday,
    #[serde(rename = "skdd")]
    pub position: String,
    #[serde(rename = "jsxm")]
    pub teacher: String,
    #[serde(rename = "kcmc")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_time_range")]
    #[serde(rename = "kcsj")]
    pub time: SpocTimeRange,
}

fn deserialize_spoc_day<'de, D>(deserializer: D) -> Result<Weekday, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    match value.as_str() {
        Some("monday") => Ok(Weekday::Monday),
        Some("tuesday") => Ok(Weekday::Tuesday),
        Some("wednesday") => Ok(Weekday::Wednesday),
        Some("thursday") => Ok(Weekday::Thursday),
        Some("friday") => Ok(Weekday::Friday),
        Some("saturday") => Ok(Weekday::Saturday),
        Some("sunday") => Ok(Weekday::Sunday),
        _ => Err(serde::de::Error::custom("Unexpected value")),
    }
}

#[derive(Debug)]
pub struct SpocTimeRange {
    pub start: PrimitiveDateTime,
    pub end: PrimitiveDateTime,
}

pub(super) fn deserialize_time_range<'de, D>(deserializer: D) -> Result<SpocTimeRange, D::Error>
where
    D: Deserializer<'de>,
{
    let format_string = format_description::parse("[year]-[month]-[day] [hour]:[minute]").unwrap();

    let s: String = Deserialize::deserialize(deserializer)?;

    let parts: Vec<&str> = s.split(' ').collect();
    if parts.len() != 2 {
        return Err(serde::de::Error::custom("Invalid time range format"));
    }

    let date_part = parts[0];
    let time_parts: Vec<&str> = parts[1].split('-').collect();
    if time_parts.len() != 2 {
        return Err(serde::de::Error::custom("Invalid time range format"));
    }

    let start_time = format!("{} {}", date_part, time_parts[0]);
    let end_time = format!("{} {}", date_part, time_parts[1]);

    let start = PrimitiveDateTime::parse(&start_time, &format_string)
        .map_err(|e| serde::de::Error::custom(e))?;
    let end = PrimitiveDateTime::parse(&end_time, &format_string)
        .map_err(|e| serde::de::Error::custom(e))?;

    Ok(SpocTimeRange { start, end })
}

impl SpocAPI {
    /// Get current week
    pub async fn get_week(&self) -> crate::Result<SpocWeek> {
        // SQL ID 似乎可以是固定值, 应该是用于鉴权的, 不知道是否会过期
        let query = r#"{"sqlid":"17275975753144ed8d6fe15425677f752c936d97de1bab76"}"#;
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryOne";
        let res = self.universal_request(query, url).await?;
        let res = serde_json::from_str::<SpocRes1>(&res)?;
        Ok(res.content)
    }

    pub async fn get_week_schedule(&self, week: &SpocWeek) -> crate::Result<Vec<SpocSchedule>> {
        // 后面三个值分别是开始日期, 结束日期和学年学期
        let query = format!(
            "{{\"sqlid\":\"17138556333937a86d7c38783bc62811e7c6bb5ef955a\",\"zksrq\":\"{}\",\"zjsrq\":\"{}\",\"xnxq\":\"{}\"}}",
            week.time.0,
            week.time.1,
            week.term
        );
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryList";
        let res = self.universal_request(&query, url).await?;
        let res = serde_json::from_str::<SpocRes2>(&res)?;
        Ok(res.content)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::env;
    use crate::Context;

    #[ignore]
    #[tokio::test]
    async fn test_spoc_get_schedule() {
        let env = env();
        let username = env.get("USERNAME").unwrap();
        let password = env.get("PASSWORD").unwrap();

        let context = Context::new();
        context.set_account(username, password);
        context.with_cookies("cookie.json");
        context.login().await.unwrap();

        let spoc = context.spoc();
        spoc.login().await.unwrap();

        let res = spoc.get_week().await.unwrap();
        println!("{:?}", res);
        let res = spoc.get_week_schedule(&res).await.unwrap();
        println!("{:?}", res);

        context.save();
    }
}
