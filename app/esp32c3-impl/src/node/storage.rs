use anyhow::Result;
use std::{cell::RefCell, collections::HashSet};

use app_core::proto::*;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};

pub struct NvsStorageService {
    nvs: RefCell<EspNvs<NvsDefault>>,
}

impl NvsStorageService {
    pub fn new(nvs: EspDefaultNvsPartition) -> Self {
        let nvs = EspNvs::new(nvs, "appstorage", true).unwrap();
        Self {
            nvs: RefCell::new(nvs),
        }
    }

    fn get_raw(&self, k: String) -> Result<Option<String>> {
        let nvs = self.nvs.borrow();
        let len = nvs.str_len(&k).unwrap_or(Some(0)).unwrap_or(0);
        Ok(if len == 0 {
            None
        } else {
            let mut buf = vec![0; len];
            nvs.get_str(&k, &mut buf)?;
            Some(String::from_utf8(buf)?)
        })
    }

    fn set_raw(&self, k: String, value: Option<String>) -> Result<()> {
        let mut nvs = self.nvs.borrow_mut();
        if let Some(v) = value {
            nvs.set_str(&k, &v)?;
        } else {
            nvs.remove(&k)?;
        }
        Ok(())
    }

    fn get(&self, k: String) -> Result<Option<String>> {
        let k = format!("data/{k}");
        self.get_raw(k)
    }

    fn set(&self, k: String, value: Option<String>) -> Result<()> {
        let k = format!("data/{k}");
        self.set_raw(k, value)?;
        Ok(())
    }

    fn list(&self) -> Result<HashSet<String>> {
        Ok(if let Some(meta) = self.get_raw("meta".into())? {
            serde_json::from_str(&meta)?
        } else {
            HashSet::new()
        })
    }
}

impl Node for NvsStorageService {
    fn node_name(&self) -> NodeName {
        NodeName::Storage
    }

    fn handle_message(
        &self,
        _ctx: std::rc::Rc<dyn Context>,
        msg: MessageWithHeader,
    ) -> HandleResult {
        if let Message::Storage(sm) = msg.body {
            return HandleResult::Finish(Message::Storage(match sm {
                StorageMessage::GetRequest(k) => {
                    let ret = self
                        .get(k)
                        .map(StorageMessage::GetResponse)
                        .map_err(|e| StorageError::Other(e.to_string()));
                    match ret {
                        Ok(x) => x,
                        Err(e) => StorageMessage::Error(e),
                    }
                }
                StorageMessage::SetRequest(k, v) => {
                    if let Err(e) = self.set(k, v) {
                        StorageMessage::Error(StorageError::Other(e.to_string()))
                    } else {
                        StorageMessage::SetResponse
                    }
                }
                StorageMessage::ListKeysRequest => match self.list() {
                    Ok(x) => StorageMessage::ListKeysResponse(x),
                    Err(e) => StorageMessage::Error(StorageError::Other(e.to_string())),
                },
                m => panic!("unexcepted message {m:?}"),
            }));
        }
        HandleResult::Discard
    }
}
