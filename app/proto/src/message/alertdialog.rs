use serde::{Deserialize, Serialize};

use crate::Bytes;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Content {
    Text(String),
    Image(Bytes),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AlertDialogMessage {
    ShowRequest(Content),
    ShowResponse,
    Close,
}
