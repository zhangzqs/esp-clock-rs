use serde::{Deserialize, Serialize};

use crate::Bytes;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlertDialogContent {
    pub text: Option<String>,
    pub image: Option<Bytes>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AlertDialogMessage {
    ShowRequest {
        duration: Option<usize>,
        content: AlertDialogContent,
    },
    ShowResponse,
    Close,
}
