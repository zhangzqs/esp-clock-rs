use crate::{ipc::StorageClient, StorageValue};

pub struct WiFiStorage(pub StorageClient);

impl WiFiStorage {
    pub fn get_ssid(&self) -> Option<String> {
        self.0.get("wifi/ssid".into()).unwrap().as_str()
    }

    pub fn get_password(&self) -> Option<String> {
        self.0.get("wifi/password".into()).unwrap().as_str()
    }

    pub fn set_ssid(&self, val: Option<String>) {
        self.0
            .set(
                "wifi/ssid".into(),
                match val {
                    Some(x) => StorageValue::String(x),
                    None => StorageValue::None,
                },
            )
            .unwrap();
    }

    pub fn set_password(&self, val: Option<String>) {
        self.0
            .set(
                "wifi/password".into(),
                match val {
                    Some(x) => StorageValue::String(x),
                    None => StorageValue::None,
                },
            )
            .unwrap();
    }
}
