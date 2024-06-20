use serde::{Deserialize, Serialize};
use time::Weekday;

use crate::StorageError;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub enum UserAlarmRingTone {
    None,
    Default,
    Music(String),
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UserAlarmRepeatCustom {
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub enum UserAlarmRepeatMode {
    /// 只响铃一次，然后自己删除
    Once,
    /// 每天
    Everyday,
    /// 仅工作日（周一至周五）
    MonToFri,
    /// 自定义每周重复类型
    Custom(UserAlarmRepeatCustom),
}

impl UserAlarmRepeatMode {
    pub fn is_active(&self, weekday: Weekday) -> bool {
        match self {
            UserAlarmRepeatMode::Once | UserAlarmRepeatMode::Everyday => true,
            UserAlarmRepeatMode::MonToFri => {
                ![Weekday::Saturday, Weekday::Sunday].contains(&weekday)
            }
            UserAlarmRepeatMode::Custom(c) => [
                (Weekday::Monday, c.monday),
                (Weekday::Tuesday, c.tuesday),
                (Weekday::Wednesday, c.wednesday),
                (Weekday::Thursday, c.thursday),
                (Weekday::Friday, c.friday),
                (Weekday::Saturday, c.saturday),
                (Weekday::Sunday, c.sunday),
            ]
            .iter()
            .any(|(d, enable)| *enable && *d == weekday),
        }
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UserAlarmBody {
    /// 闹铃
    pub ring_tone: UserAlarmRingTone,
    /// 重复模式
    pub repeat_mode: UserAlarmRepeatMode,
    /// 响铃时间 hh:mm
    pub time: (u8, u8),
    /// 闹钟响铃时将显示该备注
    pub comment: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub enum UserAlarmError {
    StorageError(StorageError),
    NotFound,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub enum UserAlarmMessage {
    Error(UserAlarmError),

    AddRequest(UserAlarmBody),
    AddResponse(usize),

    DeleteRequest(usize),
    DeleteResponse(UserAlarmBody),

    GetRequest(usize),
    GetResponse(UserAlarmBody),

    ListRequest,
    ListResponse(Vec<usize>),
}
