use serde::{Deserialize, Serialize};

use std::fs::OpenOptions;
use std::path::Path;

use crate::utils;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CredentialStore {
    pub username: Option<String>,
    pub password: Option<String>,
    /// Token for Boya API
    pub boya_token: Option<CredentialItem>,
    /// User ID for SmartClass API
    pub class_token: Option<CredentialItem>,
    /// User ID for Spoc API
    pub spoc_token: Option<CredentialItem>,
    /// Token for Srs API
    pub srs_token: Option<CredentialItem>,
    /// Mark expiration time of SSO Login Cookie.
    pub sso_login: u64,
}

impl CredentialStore {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();
        if let Ok(cred) = serde_json::from_reader(file) {
            cred
        } else {
            CredentialStore::default()
        }
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .unwrap();
        serde_json::to_writer(file, self).unwrap();
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialItem {
    pub value: String,
    pub expiration: u64,
}

impl CredentialItem {
    pub fn new(value: String, expiration: u64) -> Self {
        CredentialItem { value, expiration }
    }

    pub fn is_expired(&self) -> bool {
        self.expiration < utils::get_time_secs()
    }
}
