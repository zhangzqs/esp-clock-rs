use serde::{Deserialize, Serialize};

use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OneButtonMessage {
    // 单击
    Click,
    // 点击超过一次
    Clicks(usize),
    // 长按
    LongPressHolding(Duration),
    // 长按松手
    LongPressHeld(Duration),
}
