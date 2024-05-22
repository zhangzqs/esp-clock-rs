#[derive(Debug, Clone)]
pub enum TimeMessage {
    GetTimestampNanosRequest,
    GetTimestampNanosResponse(i128),
}
