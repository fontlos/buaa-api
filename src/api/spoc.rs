//! BUAA Spoc API

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Deserializer};

use crate::{crypto, Session, SessionError};

#[derive(Deserialize)]
struct SpocState {
    code: u32,
    msg: Option<String>
}

#[derive(Deserialize)]
struct SpocRes1 {
    content: SpocWeek
}

#[derive(Debug, Deserialize)]
pub struct SpocWeek {
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(rename = "pjmrrq")]
    pub time: (String, String),
    #[serde(rename = "mrxq")]
    pub term: String,
}

#[derive(Deserialize)]
struct SpocRes2 {
    content: Vec<SpocSchedule>
}

#[derive(Debug, Deserialize)]
pub struct SpocSchedule {
    pub weekday: String,
    #[serde(rename = "skdd")]
    pub position: String,
    #[serde(rename = "jsxm")]
    pub teacher: String,
    #[serde(rename = "strkcsj")]
    pub schedule: String,
    #[serde(rename = "kcmc")]
    pub name: String,
    #[serde(rename = "kcsj")]
    pub time: String,
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

impl Session {
    pub async fn spoc_login(&self) -> Result<String, SessionError> {
        let res = self.get("https://spoc.buaa.edu.cn/spocnewht/cas")
            .send()
            .await?;
        if res.url().as_str().contains("https://sso.buaa.edu.cn/login") {
            return Err(SessionError::LoginExpired("SSO Expired".to_string()));
        }
        let mut query = res.url().query_pairs();
        let token = match query.next() {
            Some((key, value)) => {
                if key == "token" {
                    value
                } else {
                    return Err(SessionError::LoginError("No Token".to_string()));
                }
            }
            None => return Err(SessionError::LoginError("No Token".to_string())),
        };
        // 暂时不知道有什么用, 看名字是用来刷新 token 的 token
        // let _refresh_token = match query.next() {
        //     Some((key, value)) => {
        //         if key == "refreshToken" {
        //             value
        //         } else {
        //             return Err(SessionError::LoginError("No Refresh Token".to_string()));
        //         }
        //     }
        //     None => return Err(SessionError::LoginError("No Refresh Token".to_string())),
        // };
        Ok(token.into_owned())
    }

    pub async fn spoc_universal_request(
        &self,
        query: &str,
        url: &str,
        token: &str,
    ) -> Result<String, SessionError> {
        // 逆向出来的密钥和初始向量, 既然写死了为什么不用 ECB 模式啊
        let ase_key = "inco12345678ocni";
        let ase_iv = "ocni12345678inco";
        let body = serde_json::json!({
            "param": crypto::aes::aes_encrypt_cbc(query, ase_key, ase_iv)
        });
        let token = format!("Inco-{}", token);
        let mut header = HeaderMap::new();
        header.insert(
            HeaderName::from_bytes(b"Token").unwrap(),
            HeaderValue::from_str(&token).unwrap(),
        );
        let res = self.post(url).headers(header).json(&body).send().await?;
        let res = res.text().await?;
        let status = serde_json::from_str::<SpocState>(&res)?;
        if status.code != 200 {
            return Err(SessionError::APIError(status.msg.unwrap_or("Unknown Error".to_string())));
        }
        Ok(res)
    }

    pub async fn spoc_get_week(&self, token: &str) -> Result<SpocWeek, SessionError> {
        // SQL ID 似乎可以是固定值, 应该是用于鉴权的, 不知道是否会过期
        let query = r#"{"sqlid":"17275975753144ed8d6fe15425677f752c936d97de1bab76"}"#;
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryOne";
        let res = self.spoc_universal_request(query, url, token).await?;
        let res = serde_json::from_str::<SpocRes1>(&res)?;
        Ok(res.content)
    }

    pub async fn spoc_get_week_schedule(&self, token: &str, query: &SpocWeek) -> Result<Vec<SpocSchedule>, SessionError> {
        // 后面三个值分别是开始日期, 结束日期和学年学期
        let query = format!(
            "{{\"sqlid\":\"17138556333937a86d7c38783bc62811e7c6bb5ef955a\",\"zksrq\":\"{}\",\"zjsrq\":\"{}\",\"xnxq\":\"{}\"}}",
            query.time.0,
            query.time.1,
            query.term
        );
        let url = "https://spoc.buaa.edu.cn/spocnewht/inco/ht/queryList";
        let res = self.spoc_universal_request(&query, url, token).await?;
        let res = serde_json::from_str::<SpocRes2>(&res)?;
        Ok(res.content)
    }
}

#[tokio::test]
async fn test_spoc_login() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();
    let token = session.spoc_login().await.unwrap();

    println!("{}", token);

    session.save();
}
#[tokio::test]
async fn test_spoc_universal_request() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();
    let token = session.spoc_login().await.unwrap();

    let res = session.spoc_get_week(&token).await.unwrap();
    let res = session.spoc_get_week_schedule(&token, &res).await.unwrap();
    println!("{:?}", res);
    session.save();
}