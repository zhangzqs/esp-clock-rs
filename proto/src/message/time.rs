#[derive(Debug, Clone)]
pub struct UtcDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub seconds: u8,
    pub week: u8,
}

#[derive(Debug, Clone)]
pub enum DateTimeMessage {
    UtcDateTimeRequest,
    UtcDateTimeResponse(UtcDateTime),
}
