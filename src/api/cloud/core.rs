use bytes::Bytes;
use reqwest::Method;
use serde::Serialize;

use crate::api::{Cloud, Sso};
use crate::error::Error;

use super::data::Body;

// 手动登录用 RSA 密钥, 但我们使用 SSO 登录
// From https://bhpan.buaa.edu.cn/oauth2/_next/static/chunks/pages/signin-2a57b4f57ddbb54dc27e.js
// Changed since v7 (2023.08)
// 2025.04.22
// const CLOUD_RSA_KEY: &str = "-----BEGIN PUBLIC KEY-----
// MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQC7JL0DcaMUHumSdhxXTxqiABBC
// DERhRJIsAPB++zx1INgSEKPGbexDt1ojcNAc0fI+G/yTuQcgH1EW8posgUni0mcT
// E6CnjkVbv8ILgCuhy+4eu+2lApDwQPD9Tr6J8k21Ruu2sWV5Z1VRuQFqGm/c5vaT
// OQE5VFOIXPVTaa25mQIDAQAB
// -----END PUBLIC KEY-----";
// 这个不知道是做什么的
// const CLOUD_RSA_KEY: &str = "-----BEGIN PUBLIC KEY-----
// MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA4E+eiWRwffhRIPQYvlXU
// jf0b3HqCmosiCxbFCYI/gdfDBhrTUzbt3fL3o/gRQQBEPf69vhJMFH2ZMtaJM6oh
// E3yQef331liPVM0YvqMOgvoID+zDa1NIZFObSsjOKhvZtv9esO0REeiVEPKNc+Dp
// 6il3x7TV9VKGEv0+iriNjqv7TGAexo2jVtLm50iVKTju2qmCDG83SnVHzsiNj70M
// iviqiLpgz72IxjF+xN4bRw8I5dD0GwwO8kDoJUGWgTds+VckCwdtZA65oui9Osk5
// t1a4pg6Xu9+HFcEuqwJTDxATvGAz1/YW0oUisjM0ObKTRDVSfnTYeaBsN6L+M+8g
// CwIDAQAB
// -----END PUBLIC KEY-----";

impl super::CloudApi {
    /// # Login to CloudApi
    pub async fn login(&self) -> crate::Result<()> {
        if self.cred.load().is_expired::<Sso>() {
            self.api::<Sso>().login().await?;
        }

        // TODO: 这有一个可能的 bug
        // 如果 302 到 `signin` 后可解析出登陆参数 login_challenge
        // 如果 302 到 `callback` 可以使用 refresh_token
        // 问题出在如果刷新 token 过期了会怎样, 会重定向到 `signin` 吗
        // 目前不知道其有效期
        let url =
            "https://bhpan.buaa.edu.cn/anyshare/oauth2/login?redirect=%2Fanyshare%2Fzh-cn%2Fportal";
        let res = self.client.get(url).send().await?;
        let path = res.url().path();

        if path == "/oauth2/signin" {
            // 从 URL 解析 login_challenge=xxx
            let login_challenge = res.url().query().unwrap().to_string();

            let login_challenge = cookie_store::Cookie::parse(
                login_challenge,
                &"https://bhpan.buaa.edu.cn/".parse().unwrap(),
            )
            .unwrap();

            // 这里有一条需要手动添加的临时 Cookie
            self.cookies.update(|store| {
                store
                    .insert(
                        login_challenge,
                        &"https://bhpan.buaa.edu.cn/".parse().unwrap(),
                    )
                    .unwrap();
            });
            // 发起登录请求
            let url =
                "https://sso.buaa.edu.cn/login?service=https://bhpan.buaa.edu.cn/oauth2/signin";
            let res = self.client.get(url).send().await?;
            // 移除临时 Cookie
            self.cookies.update(|store| {
                store.remove("bhpan.buaa.edu.cn", "/", "login_challenge");
            });
            // 来到回调地址证明登陆成功
            if res.url().path() != "/anyshare/oauth2/login/callback" {
                return Err(Error::server("Login failed. Redirect failed").with_label("Cloud"));
            }
        } else if path == "/anyshare/oauth2/login/callback" {
            let url = "https://bhpan.buaa.edu.cn/anyshare/oauth2/login/refreshToken";
            self.client.get(url).send().await?;
        } else {
            return Err(Error::server("Login failed. Unknown error").with_label("Cloud"));
        }

        match self
            .cookies
            .load()
            .get("bhpan.buaa.edu.cn", "/", "client.oauth2_token")
        {
            Some(t) => {
                self.cred.update(|s| {
                    s.update::<Cloud>(t.value().to_string());
                });
                // 在这里删掉 client.oauth2_(refresh_)token 以外所有 cookie
                // 防止刷新权限时干扰
                self.cookies.update(|cookies| {
                    cookies.remove("bhpan.buaa.edu.cn", "/", "ory_hydra_consent_csrf_612664744");
                    cookies.remove("bhpan.buaa.edu.cn", "/", "ory_hydra_login_csrf_612664744");
                    cookies.remove("bhpan.buaa.edu.cn", "/", "SignoutLogin3rdPartyStatus");
                    cookies.remove("bhpan.buaa.edu.cn", "/", "client.origin_uri");
                    // 这个与刷新 refresh_token 有关
                    // cookies.remove("bhpan.buaa.edu.cn", "/", "ory_hydra_session");
                    cookies.remove("bhpan.buaa.edu.cn", "/", "id_token");
                    cookies.remove("bhpan.buaa.edu.cn", "/", "state");
                    cookies.remove("bhpan.buaa.edu.cn", "/", "_csrf");
                    cookies.remove("bhpan.buaa.edu.cn", "/", "lang");
                });
                Ok(())
            }
            None => Err(Error::server("Login failed. No token").with_label("Cloud")),
        }
    }

    /// Universal Request for CloudApi (Internal)
    pub(super) async fn universal_request<'a, Q>(
        &self,
        m: Method,
        url: &str,
        body: &Body<'a, Q>,
    ) -> crate::Result<Bytes>
    where
        Q: Serialize + ?Sized,
    {
        let cred = &self.cred.load();
        if cred.is_expired::<Cloud>() {
            self.login().await?;
        }
        let token = cred.value::<Cloud>()?;

        let res = self.client.request(m, url).bearer_auth(token);

        let res = match body {
            Body::Query(f) => res.query(f),
            Body::Json(j) => res.json(j),
            Body::None => res,
        };
        let res = res.send().await?;

        let status = res.status();
        if !status.is_success() {
            let bytes = res.bytes().await?;
            let res = serde_json::from_slice::<serde_json::Value>(&bytes).unwrap_or_default();
            let cause = res
                .get("cause")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(Error::server("Operation failed")
                .with_label("Cloud")
                .with_source(format!("Status Code: {status}, Cause: {cause}")));
        }

        cred.refresh::<Cloud>();

        Ok(res.bytes().await?)
    }
}
