use serde::{Deserialize, Deserializer};

use crate::error::Error;

#[derive(Debug, Deserialize)]
pub(super) struct Res<T> {
    datas: T,
    code: String,
    msg: Option<String>,
}

impl<'de, T: Deserialize<'de>> Res<T> {
    pub(crate) fn parse(v: &'de [u8], err: &'static str) -> crate::Result<T> {
        let res: Res<T> = serde_json::from_slice(&v)?;
        if res.code == "0" {
            Ok(res.datas)
        } else {
            let source = format!(
                "Code: {}, Message: {}",
                res.code,
                res.msg.unwrap_or("Unknown error".into())
            );
            Err(Error::server(err)
                .with_label("Aas")
                .with_source(source))
        }
    }
}

// 辅助容器
pub(super) struct Data<T>(pub T);

// ====================
// 用于 get_config
// ====================

/// School calendar config
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Current week
    #[serde(rename = "classWeek")]
    pub week: u8,
    // 为什么纯数字的是学期名称, 另一个汉字的是学期代码, 什么**命名
    /// Current Term
    #[serde(rename = "xnxqmc")]
    pub term: String,
}

impl<'de> Deserialize<'de> for Data<Config> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct I {
            #[serde(rename = "welcomeInfo")]
            info: Config,
        }
        let i = I::deserialize(deserializer)?;
        Ok(Data(i.info))
    }
}