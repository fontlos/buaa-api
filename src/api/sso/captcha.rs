//! Captcha Solver

/// # Captcha Solver
pub trait CaptchaSolver: Send + Sync + 'static {
    /// # Solve Captcha
    fn solve(&self, captcha: &[u8]) -> Result<String, CaptchaError>;
}

/// # Captcha Error
#[derive(Debug)]
pub enum CaptchaError {
    /// Failed to solve captcha
    SolveFailed,
    /// User cancelled the captcha solving
    Cancelled,
}
