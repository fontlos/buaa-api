use serde::{Deserialize, Deserializer};

use crate::{Session, SessionError};

#[derive(Deserialize)]
struct BoyaStatisticWrapper {
    #[serde(deserialize_with = "deserialize_boya_statistics")]
    data: BoyaStatistic,
}

fn deserialize_boya_statistics<'de, D>(deserializer: D) -> Result<BoyaStatistic, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Intermediate {
        statistical: IntermediateBoyaStatisticData,
    }

    #[derive(Deserialize)]
    struct IntermediateBoyaStatisticData {
        #[serde(rename = "60|博雅课程")]
        data: BoyaStatistic,
    }

    let intermediate = Intermediate::deserialize(deserializer)?;

    Ok(intermediate.statistical.data)
}

#[derive(Debug, Deserialize)]
pub struct BoyaStatistic {
    #[serde(rename = "55|德育")]
    pub deyu: BoyaAssessment,
    #[serde(rename = "56|美育")]
    pub meiyu: BoyaAssessment,
    #[serde(rename = "57|劳动教育")]
    pub laoyu: BoyaAssessment,
    #[serde(rename = "58|安全健康")]
    pub anquan: BoyaAssessment,
}

#[derive(Debug, Deserialize)]
pub struct BoyaAssessment {
    #[serde(rename = "assessmentCount")]
    pub require: u8,
    #[serde(rename = "selectAssessmentCount")]
    pub select: u8,
    #[serde(rename = "completeAssessmentCount")]
    pub complete: u8,
    #[serde(rename = "failAssessmentCount")]
    pub fail: u8,
    #[serde(rename = "undoneAssessmentCount")]
    pub undone: u8,
}

impl Session {
    /// # Query Statistic
    /// - Need: [`boya_login`](#method.boya_login)
    /// - Input: Token from [`boya_login`](#method.boya_login)
    pub async fn boya_query_statistic(&self, token: &str) -> Result<BoyaStatistic, SessionError> {
        let query = "{}";
        let url = "https://bykc.buaa.edu.cn/sscv/queryStatisticByUserId";
        let res = self.boya_universal_request(query, url, token).await?;
        let res = serde_json::from_str::<BoyaStatisticWrapper>(&res)?;
        Ok(res.data)
    }
}

#[tokio::test]
async fn test_boya_query_statistic() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let mut session = Session::new_in_file("cookie.json");
    session.sso_login(&username, &password).await.unwrap();

    let token = session.boya_login().await.unwrap();
    let res = session.boya_query_statistic(&token).await.unwrap();
    println!("{:?}", res);

    session.save();
}