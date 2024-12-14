use serde::{Deserialize, Deserializer};

use crate::Context;

#[cfg(feature = "table")]
use super::query_course::{tabled_boya_kind, BoyaKind};

#[derive(Deserialize)]
struct BoyaStatistics {
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
    /// 德育
    #[serde(rename = "55|德育")]
    pub ethics: BoyaAssessment,
    /// 美育
    #[serde(rename = "56|美育")]
    pub arts: BoyaAssessment,
    /// 劳动教育
    #[serde(rename = "57|劳动教育")]
    pub labor: BoyaAssessment,
    /// 安全健康
    #[serde(rename = "58|安全健康")]
    pub safety: BoyaAssessment,
}

#[cfg(feature = "table")]
impl std::fmt::Display for BoyaStatistic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(tabled::Tabled)]
        pub struct InnerBoyaAssessment {
            #[tabled(display_with = "tabled_boya_kind")]
            pub kind: BoyaKind,
            pub require: u8,
            pub select: u8,
            pub complete: u8,
            pub fail: u8,
            pub undone: u8,
        }

        let data = vec![
            InnerBoyaAssessment {
                kind: BoyaKind::Ethics,
                require: self.ethics.require,
                select: self.ethics.select,
                complete: self.ethics.complete,
                fail: self.ethics.fail,
                undone: self.ethics.undone,
            },
            InnerBoyaAssessment {
                kind: BoyaKind::Arts,
                require: self.arts.require,
                select: self.arts.select,
                complete: self.arts.complete,
                fail: self.arts.fail,
                undone: self.arts.undone,
            },
            InnerBoyaAssessment {
                kind: BoyaKind::Labor,
                require: self.labor.require,
                select: self.labor.select,
                complete: self.labor.complete,
                fail: self.labor.fail,
                undone: self.labor.undone,
            },
            InnerBoyaAssessment {
                kind: BoyaKind::Safety,
                require: self.safety.require,
                select: self.safety.select,
                complete: self.safety.complete,
                fail: self.safety.fail,
                undone: self.safety.undone,
            },
        ];

        let table = crate::utils::table(&data);
        write!(f, "{}", table)
    }
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

impl Context {
    /// # Query Statistic
    /// - Need: [`boya_login`](#method.boya_login)
    pub async fn boya_query_statistic(&self) -> crate::Result<BoyaStatistic> {
        let query = "{}";
        let url = "https://bykc.buaa.edu.cn/sscv/queryStatisticByUserId";
        let res = self.boya_universal_request(query, url).await?;
        let res = serde_json::from_str::<BoyaStatistics>(&res)?;
        Ok(res.data)
    }
}

#[tokio::test]
async fn test_boya_query_statistic() {
    let env = crate::utils::env();
    let username = env.get("USERNAME").unwrap();
    let password = env.get("PASSWORD").unwrap();

    let session = Context::new();
    session.with_cookies("cookie.json");

    session.sso_login(&username, &password).await.unwrap();

    session.boya_login().await.unwrap();
    let res = session.boya_query_statistic().await.unwrap();

    println!("{:?}", res);

    session.save();
}
