use std::{collections::HashSet, rc::Rc};

use app_core::proto::{
    Context, HandleResult, Message, MessageWithHeader, Node, NodeName, StorageError,
    StorageMessage, StorageValue,
};

pub struct LocalStorageService {
    stg: web_sys::Storage,
}

impl LocalStorageService {
    pub fn new() -> Self {
        let stg = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        Self { stg }
    }

    fn get_raw(&self, key: &str) -> Result<Option<String>, StorageError> {
        self.stg
            .get(key)
            .map_err(|e| StorageError::IOError(format!("{e:?}")))
    }

    fn set_raw(&self, key: &str, value: Option<String>) -> Result<(), StorageError> {
        if let Some(value) = value {
            self.stg
                .set(key, &value)
                .map_err(|e| StorageError::IOError(format!("{e:?}")))?;
            self.add_list(key)?;
        } else {
            self.stg
                .remove_item(key)
                .map_err(|e| StorageError::IOError(format!("{e:?}")))?;
            self.remove_list(key)?;
        }
        Ok(())
    }

    fn get(&self, key: &str) -> Result<StorageValue, StorageError> {
        Ok(match self.get_raw(&format!("data/{key}"))? {
            Some(x) => {
                serde_json::from_str(&x).map_err(|e| StorageError::TypeError(format!("{e:?}")))?
            }
            None => StorageValue::None,
        })
    }

    fn set(&self, key: &str, value: StorageValue) -> Result<(), StorageError> {
        self.set_raw(
            &format!("data/{key}"),
            match value {
                StorageValue::None => None,
                x => Some(
                    serde_json::to_string(&x)
                        .map_err(|e| StorageError::TypeError(format!("{e:?}")))?,
                ),
            },
        )
    }

    fn list(&self) -> Result<HashSet<String>, StorageError> {
        Ok(self
            .get_raw("list_meta")?
            .map(|s| serde_json::from_str::<HashSet<String>>(&s).unwrap())
            .unwrap_or_default())
    }

    fn add_list(&self, key: &str) -> Result<(), StorageError> {
        let mut list = self.list()?;
        list.insert(key.into());
        self.set_raw("list_meta", Some(serde_json::to_string(&list).unwrap()))?;
        Ok(())
    }

    fn remove_list(&self, key: &str) -> Result<(), StorageError> {
        let mut list = self.list()?;
        if list.remove(key) {
            self.set_raw("list_meta", Some(serde_json::to_string(&list).unwrap()))?;
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
                StorageMessage::SetRequest(key, value) => {
                    self.set(&key, value).map(|_| StorageMessage::SetResponse)
                }
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
