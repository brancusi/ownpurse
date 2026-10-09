//! Errors with the CLI's codes: `needs_login`, `api_error`, `daily_limit`,
//! `rate_limited`, `forbidden`, `not_found`, `usage`, `integrity`, each with a hint and, for HTTP errors, the
//! first 1,500 characters of the response body.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
    pub hints: Vec<String>,
    pub body: Option<String>,
}

impl ApiError {
    pub fn new(
        code: &'static str,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> ApiError {
        let hint = hint.into();
        ApiError {
            code,
            message: message.into(),
            hints: if hint.is_empty() { vec![] } else { vec![hint] },
            body: None,
        }
    }
    pub fn needs_login(message: impl Into<String>, hint: impl Into<String>) -> ApiError {
        ApiError::new("needs_login", message, hint)
    }
    pub fn api(message: impl Into<String>, hint: impl Into<String>) -> ApiError {
        ApiError::new("api_error", message, hint)
    }
    pub fn usage(message: impl Into<String>, hint: impl Into<String>) -> ApiError {
        ApiError::new("usage", message, hint)
    }
    pub fn with_hint(mut self, hint: impl Into<String>) -> ApiError {
        self.hints.push(hint.into());
        self
    }
    pub fn with_body(mut self, body: impl Into<String>) -> ApiError {
        self.body = Some(body.into());
        self
    }
    pub fn code(&self) -> &'static str {
        self.code
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    /// The first hint (most errors carry one).
    pub fn hint(&self) -> &str {
        self.hints.first().map(String::as_str).unwrap_or("")
    }
    /// The exit code: 2 for usage errors, 1 otherwise.
    pub fn exit_code(&self) -> i32 {
        if self.code == "usage" { 2 } else { 1 }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApiError {}
