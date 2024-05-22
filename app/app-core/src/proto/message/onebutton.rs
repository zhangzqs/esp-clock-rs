use std::time::Duration;

#[derive(Debug, Clone)]
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
