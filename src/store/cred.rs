use serde::{Deserialize, Serialize};

use std::fs::OpenOptions;
use std::path::Path;

use crate::utils;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CredentialStore {
    pub username: Option<String>,
    pub password: Option<String>,
    /// Token for Boya API
    pub boya_token: CredentialItem,
    /// Token for Class API
    pub class_token: CredentialItem,
    /// Token for Cloud API
    pub cloud_token: CredentialItem,
    /// Token for Spoc API
    pub spoc_token: CredentialItem,
    /// Token for Srs API
    pub srs_token: CredentialItem,
    /// Mark login expiration time of SSO
    pub sso: CredentialItem,
}

impl CredentialStore {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let file = OpenOptions::new().read(true).open(path).unwrap();
        serde_json::from_reader(file).unwrap_or_default()
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .unwrap();
        serde_json::to_writer(file, self).unwrap();
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CredentialItem {
    value: Option<String>,
    expiration: u64,
}

impl CredentialItem {
    pub fn value(&self) -> Option<&String> {
        self.value.as_ref()
    }

    pub fn set(&mut self, value: String, expiration: u64) {
        self.value = Some(value);
        self.expiration = utils::get_time_secs() + expiration;
    }

    pub fn refresh(&mut self, expiration: u64) {
        self.expiration = utils::get_time_secs() + expiration;
    }

    pub fn is_expired(&self) -> bool {
        self.expiration < utils::get_time_secs()
    }
}
