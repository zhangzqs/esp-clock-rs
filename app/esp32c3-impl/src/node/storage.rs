use anyhow::Result;
use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use app_core::proto::*;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};

pub struct NvsStorageService {
    nvs: RefCell<EspNvs<NvsDefault>>,
    index: RefCell<HashMap<String, u16>>,
}

impl NvsStorageService {
    pub fn new(nvs: EspDefaultNvsPartition) -> Self {
        let nvs = EspNvs::new(nvs, "appstorage", true).unwrap();
        let ret = Self {
            nvs: RefCell::new(nvs),
            index: RefCell::new(HashMap::new()),
        };
        ret.load_meta();
        ret
    }

    fn get_raw(&self, k: String) -> Result<Option<String>> {
        let nvs = self.nvs.borrow();
        let strlen = nvs.str_len(&k).unwrap_or(Some(0)).unwrap_or(0);
        Ok(if strlen == 0 {
            None
        } else {
            let mut buf = vec![0; strlen];
            nvs.get_str(&k, &mut buf)?;
            buf.remove(buf.len() - 1);
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
        let idx = self.index.borrow()[&k];
        self.get_raw(idx.to_string())
    }

    fn gen_next_idx(&self) -> u16 {
        self.index.borrow().values().copied().max().unwrap_or(0) + 1
    }

    fn set(&self, k: String, value: Option<String>) -> Result<()> {
        if !self.index.borrow().contains_key(&k) {
            let idx = self.gen_next_idx();
            self.index.borrow_mut().insert(k.clone(), idx);
        }
        let idx = self.index.borrow()[&k];
        self.set_raw(idx.to_string(), value)?;
        self.store_meta()?;
        Ok(())
    }

    fn list(&self) -> Result<HashSet<String>> {
        Ok(self.index.borrow().keys().cloned().collect())
    }

    fn load_meta(&self) {
        if let Some(x) = self.get_raw("0".to_string()).unwrap() {
            *self.index.borrow_mut() = serde_json::from_str(&x).unwrap_or_default();
        }
    }

    fn store_meta(&self) -> Result<()> {
        self.set_raw(
            "0".to_string(),
            Some(serde_json::to_string(&*self.index.borrow())?),
        )?;
        Ok(())
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
