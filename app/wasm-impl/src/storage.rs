use std::{collections::HashSet, rc::Rc};

use app_core::proto::{
    Context, HandleResult, Message, MessageWithHeader, Node, NodeName, StorageError, StorageMessage,
};
use wasm_bindgen::JsValue;

pub struct LocalStorageService {
    stg: web_sys::Storage,
}

impl LocalStorageService {
    pub fn new() -> Self {
        let stg = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        Self { stg }
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, JsValue> {
        let k = format!("data/{key}");
        self.stg.get(&k)
    }

    pub fn set(&self, key: &str, value: Option<&str>) -> Result<(), JsValue> {
        let k = format!("data/{key}");
        if let Some(value) = value {
            self.stg.set(&k, value)?;
            self.add_list(key)?;
        } else {
            self.stg.remove_item(&k)?;
            self.remove_list(key)?;
        }
        Ok(())
    }

    pub fn list(&self) -> Result<HashSet<String>, JsValue> {
        Ok(self
            .stg
            .get("list_meta")?
            .map(|s| serde_json::from_str::<HashSet<String>>(&s).unwrap())
            .unwrap_or_default())
    }

    pub fn add_list(&self, key: &str) -> Result<(), JsValue> {
        let mut list = self.list()?;
        list.insert(key.into());
        self.stg
            .set("list_meta", &serde_json::to_string(&list).unwrap())?;
        Ok(())
    }

    pub fn remove_list(&self, key: &str) -> Result<(), JsValue> {
        let mut list = self.list()?;
        if list.remove(key) {
            self.stg
                .set("list_meta", &serde_json::to_string(&list).unwrap())?;
        }
        Ok(())
    }
}

impl Node for LocalStorageService {
    fn node_name(&self) -> NodeName {
        NodeName::Storage
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        if let Message::Storage(sm) = msg.body {
            let resp = match sm {
                StorageMessage::GetRequest(key) => self.get(&key).map(StorageMessage::GetResponse),
                StorageMessage::SetRequest(key, value) => self
                    .set(&key, value.as_deref())
                    .map(|_| StorageMessage::SetResponse),
                StorageMessage::ListKeysRequest => {
                    self.list().map(StorageMessage::ListKeysResponse)
                }
                m => panic!("unexpected message {:?}", m),
            };
            match resp {
                Ok(v) => {
                    return HandleResult::Finish(Message::Storage(v));
                }
                Err(e) => {
                    return HandleResult::Finish(Message::Storage(StorageMessage::Error(
                        StorageError::Other(format!("{:?}", e)),
                    )));
                }
            }
        }
        HandleResult::Discard
    }
}
