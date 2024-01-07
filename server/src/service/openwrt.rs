use poem_openapi::{payload::Json, OpenApi};
use serde::Deserialize;
use serde_json::json;

mod client;

#[derive(Debug, Deserialize)]
pub struct OpenWrtServiceConfig {
    client: client::ClientConfig,
}

pub struct OpenWrt {
    client: client::Client,
}

impl OpenWrt {
    pub fn new(config: OpenWrtServiceConfig) -> Self {
        OpenWrt {
            client: client::Client::new(config.client),
        }
    }
}

#[OpenApi]
impl OpenWrt {
    #[oai(path = "/openwrt/network_info", method = "get")]
    async fn network_info(&self) -> Json<serde_json::Value> {
        let info = self.client.get_all_client_network_info().await;
        let text = format!("{:?}", info);
        Json(json!({
                "text": text,
        }))
    }
}
