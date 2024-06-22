use serde::{Deserialize, Serialize};

use crate::Image;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotifactionContent {
    pub title: Option<String>,
    pub text: Option<String>,
    pub icon: Option<Image>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NotifactionMessage {
    ShowRequest {
        /// 持续时间，0表示永久
        #[serde(default)]
        duration: usize,

        /// 内容
        content: NotifactionContent,
    },
    ShowResponse,
    Close,
}
