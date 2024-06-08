use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use app_core::proto::*;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};

#[derive(Debug, Deserialize, Serialize)]
enum ItemType {
    None,
    String,
    Blob,
}

fn m(v: &StorageValue) -> ItemType {
    match v {
        StorageValue::None => ItemType::None,
        StorageValue::Bytes(_) => ItemType::Blob,
        StorageValue::String(_) => ItemType::String,
    }
}

pub struct NvsStorageService {
    nvs: RefCell<EspNvs<NvsDefault>>,
    index: RefCell<HashMap<String, (u16, ItemType)>>,
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

    fn set_raw_blob(&self, k: String, v: &[u8]) -> Result<()> {
        let mut nvs = self.nvs.borrow_mut();
        nvs.set_blob(&k, v)?;
        Ok(())
    }

    fn get_raw_blob(&self, k: String) -> Result<Option<Vec<u8>>> {
        let nvs = self.nvs.borrow();
        Ok(if let Some(x) = nvs.blob_len(&k)? {
            let mut v = vec![0; x];
            nvs.get_blob(&k, &mut v)?;
            Some(v)
        } else {
            None
        })
    }

    fn get_raw_str(&self, k: String) -> Result<Option<String>> {
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

    fn set_raw_str(&self, k: String, value: String) -> Result<()> {
        let mut nvs = self.nvs.borrow_mut();
        nvs.set_str(&k, &value)?;
        Ok(())
    }

    fn remove_raw(&self, k: String) -> Result<()> {
        let mut nvs = self.nvs.borrow_mut();
        nvs.remove(&k)?;
        Ok(())
    }

    fn get(&self, k: String) -> Result<StorageValue> {
        if !self.index.borrow().contains_key(&k) {
            return Ok(StorageValue::None);
        }
        let (idx, typ) = &self.index.borrow()[&k];
        Ok(match typ {
            ItemType::None => StorageValue::None,
            ItemType::String => {
                if let Some(x) = self.get_raw_str(idx.to_string())? {
                    StorageValue::String(x)
                } else {
                    StorageValue::None
                }
            }
            ItemType::Blob => {
                if let Some(x) = self.get_raw_blob(idx.to_string())? {
                    StorageValue::Bytes(Bytes(x))
                } else {
                    StorageValue::None
                }
            }
        })
    }

    fn gen_next_idx(&self) -> u16 {
        self.index.borrow().values().map(|x| x.0).max().unwrap_or(0) + 1
    }

    fn set(&self, k: String, value: StorageValue) -> Result<()> {
        if !self.index.borrow().contains_key(&k) {
            let idx = self.gen_next_idx();
            self.index.borrow_mut().insert(k.clone(), (idx, m(&value)));
        }
        let (idx, _) = &self.index.borrow()[&k];
        match value {
            StorageValue::None => self.remove_raw(idx.to_string()),
            StorageValue::Bytes(Bytes(x)) => self.set_raw_blob(idx.to_string(), &x),
            StorageValue::String(x) => self.set_raw_str(idx.to_string(), x),
        }?;
        self.store_meta()?;
        Ok(())
    }

    fn list(&self, prefix: String) -> Result<HashSet<String>> {
        Ok(self
            .index
            .borrow()
            .keys()
            .filter(|x| x.starts_with(&prefix))
            .map(|x| x.into())
            .collect())
    }

    fn load_meta(&self) {
        if let Some(x) = self.get_raw_str("0".to_string()).unwrap() {
            *self.index.borrow_mut() = serde_json::from_str(&x).unwrap_or_default();
        }
    }

    fn store_meta(&self) -> Result<()> {
        self.set_raw_str(
            "0".to_string(),
            serde_json::to_string(&*self.index.borrow())?,
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
                StorageMessage::ListKeysRequest(prefix) => match self.list(prefix) {
                    Ok(x) => StorageMessage::ListKeysResponse(x),
                    Err(e) => StorageMessage::Error(StorageError::Other(e.to_string())),
                },
                m => panic!("unexcepted message {m:?}"),
            }));
        }
        HandleResult::Discard
    }
}
