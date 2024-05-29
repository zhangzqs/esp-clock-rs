use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OneButtonMessage {
    // 单击
    Click,
    // 点击超过一次
    Clicks(usize),
    // 长按持续的毫秒数
    LongPressHolding(usize),
    // 长按松手
    LongPressHeld(usize),
}
