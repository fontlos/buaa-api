use bytes::Bytes;
use futures::channel::oneshot;
use log::{trace, warn};
use reqwest::{RequestBuilder, Response, StatusCode};

use std::sync::Arc;

use crate::api::sso::captcha::CaptchaAction;
use crate::api::{Core, Sso, Vpn};
use crate::error::Error;
use crate::utils;

impl super::SsoApi {
    /// # Login to SSO
    pub async fn login(&self) -> crate::Result<()> {
        let cred = self.cred.load();
        let login_url = "https://sso.buaa.edu.cn/login";
        let verify_url = "https://uc.buaa.edu.cn/";
        self.login_inner(login_url, verify_url).await?;
        cred.refresh::<Sso>();
        Ok(())
    }

    // 考虑到常用除了 class 都能直连, 那么 VPN 模式其实没那么重要, 让需要的情况自己调用
    // 2026.11.08 对于 VPN 登录似乎强制使用验证码了
    /// # Login to SSO via VPN
    ///
    /// **Note**: Currently only for ClassAPI
    pub async fn login_vpn(&self) -> crate::Result<()> {
        let cred = self.cred.load();
        let login_url = "https://d.buaa.edu.cn/https/77726476706e69737468656265737421e3e44ed225256951300d8db9d6562d/login?service=https%3A%2F%2Fd.buaa.edu.cn%2Flogin%3Fcas_login%3Dtrue";
        let verify_url = "https://d.buaa.edu.cn/";
        self.login_inner(login_url, verify_url).await?;
        cred.refresh::<Vpn>();
        Ok(())
    }

    async fn login_inner(&self, login_url: &str, verify_url: &str) -> crate::Result<()> {
        // 检查登录有效性
        let res = self.client.get(login_url).send().await?;
        // 重定向到这里说明 Cookie 有效, 但无法刷新
        if res.url().as_str() == verify_url {
            trace!("SSO still valid");
            return Ok(());
        }

        // 重新登录
        let req = self.prepare_login(res, login_url).await?;
        // 如果返回 None, 说明此时在非校园网环境, 并且连接错误, 那么我们就只能认为登陆成功了
        // 如果有返回响应体, 我们再根据响应体判断
        let res = match self.finish_login(req).await? {
            Some(r) => r,
            None => return Ok(()),
        };
        // 只有状态码是有效信息, 如果成功就是成功, 如果 401, 那么除了账号密码错误还可能是账号有风险
        let status = res.status();
        if status.is_success() {
            return Ok(());
        }

        // 当 "账号存在安全风险", "您的密码已过期或是已知的弱密码" 时, 上面会返回 401, 此时尝试忽略风险继续登录
        if let Some(req) = self.prepare_try_ignore_risk(res, login_url).await? {
            warn!(
                "SSO account has security risk, try to ignore and continue. Suggest change password."
            );
            let res = match self.finish_login(req).await? {
                Some(r) => r,
                None => return Ok(()),
            };
            // 只有状态码是有效信息, 如果成功就是成功, 如果 401, 那么除了账号密码错误还可能是账号有风险
            let status = res.status();
            if status.is_success() {
                return Ok(());
            }
        }

        // 其他情况直接返回错误
        // TODO: 如果没注册验证码处理器但登录需要验证码可能也会触发这个错误
        Err(login_failed(status))
    }

    /// 准备尝试一次登录, 如果 401 有可能是因为账号有风险, 需要忽略风险继续登录
    async fn prepare_login(&self, res: Response, login_url: &str) -> crate::Result<RequestBuilder> {
        let cred = self.cred.load();
        let un = cred.username()?;
        let pw = cred.password()?;

        // 重新登录
        let bytes = res.bytes().await?;
        let execution = parse_execution(&bytes)?;
        let captcha = self.solve_captcha(&bytes).await?;
        // 完成登录
        let form = [
            ("username", un),
            ("password", pw),
            ("captcha", &captcha),
            ("submit", "登录"),
            ("type", "username_password"),
            ("execution", execution),
            ("_eventId", "submit"),
        ];
        Ok(self.client.post(login_url).form(&form))
    }

    /// VPN 模式下可能需要验证码
    ///
    /// TODO: 目前只在验证码处理器存在时才处理验证码, 否则直接返回空字符串
    async fn solve_captcha(&self, bytes: &Bytes) -> crate::Result<String> {
        // VPN 模式可能需要验证码, 验证码配置写死在 JS 对象里, 直接解析字符串
        // config.captcha = {
        //     type: 'image',
        //     id: 'xxx'
        // };
        let mut captcha = String::new();
        if let Some(solver) = &self.captcha
            && let Some(config) = utils::parse_by_tag(&bytes, "config.captcha =", ";")
        {
            let captcha_id = match utils::parse_by_tag(config.as_bytes(), "id: '", "'") {
                Some(s) => s,
                None => {
                    return Err(Error::server("Login failed. No Captcha ID").with_label("Sso"));
                }
            };

            // 循环来做到刷新验证码, 每次收到刷新请求时重新执行
            loop {
                // 时间戳参数 t(ms) 加不加都可以
                let url = format!(
                    "https://d.buaa.edu.cn/https/77726476706e69737468656265737421e3e44ed225256951300d8db9d6562d/captcha?vpn-1&captchaId={}&t={}",
                    captcha_id,
                    utils::time::DateTime::millis()
                );
                let img = self.client.get(&url).send().await?.bytes().await?;

                // 为了减少不必要的麻烦, 对于验证码这种冷路径直接拉起一个线程应该比线程池更轻量
                let solver = Arc::clone(solver);
                let img_vec = img.to_vec();
                let (tx, rx) = oneshot::channel();
                std::thread::spawn(move || {
                    let _ = tx.send(solver.solve(&img_vec));
                });

                // 异步等待, 让执行器线程让出异步线程
                let action = rx
                    .await
                    .map_err(|_| Error::io("Captcha solver thread panicked").with_label("Sso"))?
                    .map_err(|e| Error::auth(e).with_label("Sso"))?;

                match action {
                    CaptchaAction::Submit(value) => {
                        captcha = value;
                        break;
                    }
                    CaptchaAction::Refresh => continue,
                    CaptchaAction::Cancel => {
                        return Err(Error::auth("Captcha solving cancelled").with_label("Sso"));
                    }
                }
            }
        }
        Ok(captcha)
    }

    /// 尝试忽略安全风险并继续登录
    ///
    /// 返回 Some 表示确实可以忽略风险登录, 返回 None 表示这不是安全风险导致的, 无法继续登录
    async fn prepare_try_ignore_risk(
        &self,
        res: Response,
        login_url: &str,
    ) -> crate::Result<Option<RequestBuilder>> {
        let bytes = res.bytes().await?;
        if bytes
            .windows(b"continueForm".len())
            .any(|w| w == b"continueForm")
        {
            let execution = parse_execution(&bytes)?;
            let form = [("execution", execution), ("_eventId", "ignoreAndContinue")];
            let req = self.client.post(login_url).form(&form);
            return Ok(Some(req));
        }
        Ok(None)
    }

    /// 完成登录
    ///
    /// 返回 Some 代表一切正常, 返回 None 代表在非校园网环境最后一步请求错误, 我们暂时认为登录成功
    async fn finish_login(&self, req: RequestBuilder) -> crate::Result<Option<Response>> {
        match req.send().await {
            Ok(r) => Ok(Some(r)),
            // 2026.11.12 目前在非校园网环境下 UC 会触发 ERR_HTTP2_PROTOCOL_ERROR
            // 如果此时 SSO 最后确实会转跳到 UC, 那么实际上凭据是有效的, 但是因为在协议层就发生了错误, 难以跟踪转跳
            // TODO: 因此我们在这里临时接受如果当前处于非校园网环境, 忽略由于请求连接导致的错误
            Err(e) if !self.api::<Core>().on_campus() && e.is_request() => {
                warn!(
                    "Not on campus network. SSO login may succeed, but connection error occurred."
                );
                Ok(None)
            }
            Err(e) => Err(Error::server("Login failed.")
                .with_label("Sso")
                .with_source(e)),
        }
    }
}

/// 解析登录页 execution 值
fn parse_execution(bytes: &Bytes) -> crate::Result<&str> {
    match utils::parse_by_tag(&bytes, "\"execution\" value=\"", "\"") {
        Some(s) => Ok(s),
        None => Err(Error::server("Login failed. No Execution Value").with_label("Sso")),
    }
}

/// 登录失败
fn login_failed(status: StatusCode) -> crate::Error {
    let source = format!("Response Code: {}", status);
    Error::server("Login failed. Maybe wrong username or password")
        .with_label("Sso")
        .with_source(source)
}
