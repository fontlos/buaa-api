/// For Boya generic requests, the ones obtained backwards, hard-coded into JS, should theoretically be forever the same.
/// 2025.04.22
pub const BOYA_RSA_KEY: &str = "-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDlHMQ3B5GsWnCe7Nlo1YiG/YmH
dlOiKOST5aRm4iaqYSvhvWmwcigoyWTM+8bv2+sf6nQBRDWTY4KmNV7DBk1eDnTI
Qo6ENA31k5/tYCLEXgjPbEjCK9spiyB62fCT6cqOhbamJB0lcDJRO6Vo1m3dy+fD
0jbxfDVBBNtyltIsDQIDAQAB
-----END PUBLIC KEY-----";

/// For class login, the key comes from the reverse analysis of JS
/// 2025.04.22
pub const CLASS_DES_KEY: &str = "Jyd#351*";
