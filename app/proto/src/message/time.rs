use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TimeMessage {
    GetTimestampNanosRequest,
    GetTimestampNanosResponse(i128),
}
