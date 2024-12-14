//! Boya Course API

pub mod query_course;
pub mod query_selected;
pub mod query_statistic;
mod util;

use std::sync::Arc;
use std::ops::Deref;

use reqwest::Client;
use serde::Deserialize;

use crate::{SharedResources, Context};

pub struct BoyaAPI {
    pub context: Arc<SharedResources>,
}

#[derive(Deserialize)]
struct BoyaStatus {
    status: String,
    errmsg: String,
}

impl Context {
    pub fn boya(&self) -> BoyaAPI {
        BoyaAPI {
            context: Arc::clone(&self.shared),
        }
    }
}

impl Deref for BoyaAPI {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.context.client
    }
}
