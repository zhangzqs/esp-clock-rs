use std::fmt::Display;

use poem::error::ResponseError;
use reqwest::StatusCode;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub struct ApiError {
    pub code: u16,
    pub msg: Option<String>,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.msg {
            write!(f, "{}", msg)
        } else {
            write!(f, "ApiError({})", self.code)
        }
    }
}

impl ResponseError for ApiError {
    fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn as_response(&self) -> poem::Response
    where
        Self: std::error::Error + Send + Sync + 'static,
    {
        poem::Response::builder()
            .status(self.status())
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "error": self.to_string(),
                })
                .to_string(),
            )
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        Self {
            code: err.status().map_or(500, |s| s.as_u16()),
            msg: Some(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self {
            code: 500,
            msg: Some(err.to_string()),
        }
    }
}
