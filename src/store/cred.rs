use serde::{Deserialize, Serialize};

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

    /// Mark expiration time of Boya Login Cookie.
    pub boya_login: Option<u64>,
    /// Mark expiration time of SmartClass Login Cookie.
    pub class_login: Option<u64>,
    /// Mark expiration time of Spoc Login Cookie.
    pub spoc_login: Option<u64>,
    /// Mark expiration time of Srs Login Cookie.
    pub srs_login: Option<u64>,
    /// Mark expiration time of SSO Login Cookie.
    pub sso_login: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialItem {
    pub value: String,
    pub expiration: u64,
}
