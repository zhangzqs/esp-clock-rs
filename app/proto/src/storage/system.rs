use crate::{ipc::StorageClient, StorageValue};

pub struct SystemStorage(pub StorageClient);

impl SystemStorage {
    pub fn get_monitor_enable(&self) -> bool {
        self.0
            .get("system/monitor-enable".into())
            .map(|x| match x {
                StorageValue::String(x) => &x == "1",
                _ => false,
            })
            .unwrap_or_default()
    }

    pub fn set_monitor_enable(&self, enable: bool) {
        self.0
            .set(
                "system/monitor-enable".into(),
                StorageValue::String(if enable { "1" } else { "0" }.into()),
            )
            .unwrap();
    }
}
