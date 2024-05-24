use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiStorageConfiguration {
    pub ssid: String,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetIpInfo {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WiFiMessage {
    // 获取本地存储中的wifi链接配置
    GetStorageWiFiConfigurationRequest,
    GetStorageWiFiConfigurationResponse(Option<WiFiStorageConfiguration>),

    // 根据指定配置连接wifi
    ConnectRequest(WiFiStorageConfiguration),
    ConnectResponse,

    // 获取ip信息
    GetIpInfoRequest,
    GetIpInfoResponse(NetIpInfo),
}
