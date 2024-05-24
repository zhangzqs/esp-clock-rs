use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuzzerMessage {
    Tone(u32),
}
