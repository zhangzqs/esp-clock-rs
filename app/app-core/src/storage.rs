use proto::ipc::StorageClient;

pub struct WiFiStorage(pub StorageClient);

impl WiFiStorage {
    pub fn get_ssid(&self) -> Option<String> {
        self.0.get("wifi/ssid".into()).unwrap().as_str("").ok()
    }

    pub fn get_password(&self) -> Option<String> {
        self.0.get("wifi/password".into()).unwrap().as_str("").ok()
    }
}
