//! Captcha Solver

/// # Captcha Action
pub enum CaptchaAction {
    /// Submit the captcha solution
    Submit(String),
    /// Refresh the captcha image
    Refresh,
    /// Cancel the captcha solving
    Cancel,
}

/// # Captcha Solver
pub trait CaptchaSolver: Send + Sync + 'static {
    /// # Solve Captcha
    fn solve(&self, captcha: &[u8]) -> Result<CaptchaAction, String>;
}
