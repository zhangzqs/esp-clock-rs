use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawLineInput {
    start: (u16, u16),
    end: (u16, u16),
    color: (u8, u8, u8),
    width: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawRectangleInput {
    top_left: (u16, u16),
    size: (u16, u16),
    color: (u8, u8, u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawCircleInput {
    center: (u16, u16),
    radius: (u16, u16),
    color: (u8, u8, u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawPixelsInput {
    top_left: (u16, u16),
    width: u16,
    pixels: Vec<(u8, u8, u8)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CanvasMessage {
    Open,
    Close,
    Clear((u8, u8, u8)),
    DrawLine(DrawLineInput),
    DrawCircle(DrawCircleInput),
    DrawRectangle(DrawRectangleInput),
    DrawPixels(DrawPixelsInput),
}
