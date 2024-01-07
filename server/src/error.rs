use poem::error::ResponseError;
use reqwest::StatusCode;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("reource not found: {resource:?}")]
    NotFound { resource: Option<String> },
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("other error")]
    OtherError {
        code: Option<u16>,
        msg: Option<String>,
    },
}

impl ResponseError for ApiError {
    fn status(&self) -> StatusCode {
        match self {
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::Reqwest(err) => err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            ApiError::SerdeJson(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::OtherError { code, .. } => code
                .map(|x| StatusCode::from_u16(x).unwrap())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        }
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
