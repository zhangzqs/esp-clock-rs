use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiStorageConfiguration {
    pub ssid: String,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetIpInfo {
    pub ip: Ipv4Addr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WiFiError {
    NotStarted,
    NotFoundAP,
    ApNeedPassword,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WiFiMessage {
    Error(WiFiError),

    // 开启WiFi热点
    StartAPRequest,
    StartAPResponse,

    // 根据指定配置连接wifi
    ConnectRequest(WiFiStorageConfiguration),
    ConnectResponse,

    // 获取ip信息
    GetIpInfoRequest,
    GetIpInfoResponse(NetIpInfo),

    ConnectedBroadcast,
    APStartedBroadcast,
}
