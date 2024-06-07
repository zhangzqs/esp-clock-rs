use std::{collections::HashSet, rc::Rc};

use crate::{Context, Message, NodeName};

use crate::message::{StorageError, StorageMessage, StorageValue};
#[derive(Clone)]
pub struct StorageClient(pub Rc<dyn Context>);

impl StorageClient {
    pub fn set(&self, key: String, value: StorageValue) -> Result<(), StorageError> {
        let r = self.0.sync_call(
            NodeName::Storage,
            Message::Storage(StorageMessage::SetRequest(key, value)),
        );
        match r.unwrap() {
            Message::Storage(StorageMessage::SetResponse) => Ok(()),
            Message::Storage(StorageMessage::Error(e)) => Err(e),
            m => panic!("unexpected message {:?}", m),
        }
    }
    pub fn get(&self, key: String) -> Result<StorageValue, StorageError> {
        let r = self.0.sync_call(
            NodeName::Storage,
            Message::Storage(StorageMessage::GetRequest(key)),
        );
        match r.unwrap() {
            Message::Storage(StorageMessage::GetResponse(r)) => Ok(r),
            Message::Storage(StorageMessage::Error(e)) => Err(e),
            m => panic!("unexpected message {:?}", m),
        }
    }

    pub fn list(&self, prefix: String) -> Result<HashSet<String>, StorageError> {
        let r = self.0.sync_call(
            NodeName::Storage,
            Message::Storage(StorageMessage::ListKeysRequest(prefix)),
        );
        match r.unwrap() {
            Message::Storage(StorageMessage::ListKeysResponse(r)) => Ok(r),
            Message::Storage(StorageMessage::Error(e)) => Err(e),
            m => panic!("unexpected message {:?}", m),
        }
    }
}
