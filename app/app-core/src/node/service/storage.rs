use std::{cell::RefCell, collections::HashMap};

use crate::proto::*;

pub struct MockStorageService {
    data: RefCell<HashMap<String, StorageValue>>,
}

impl MockStorageService {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(HashMap::new()),
        }
    }
}

impl Node for MockStorageService {
    fn node_name(&self) -> NodeName {
        NodeName::Storage
    }

    fn handle_message(
        &self,
        _ctx: std::rc::Rc<dyn Context>,
        msg: MessageWithHeader,
    ) -> HandleResult {
        if let Message::Storage(sm) = msg.body {
            let mut data = self.data.borrow_mut();
            return HandleResult::Finish(Message::Storage(match sm {
                StorageMessage::GetRequest(k) => {
                    StorageMessage::GetResponse(data.get(&k).cloned().unwrap_or(StorageValue::None))
                }
                StorageMessage::SetRequest(k, v) => {
                    match v {
                        StorageValue::None => {
                            data.remove(&k);
                        }
                        v => {
                            data.insert(k, v);
                        }
                    }
                    StorageMessage::SetResponse
                }
                StorageMessage::ListKeysRequest(prefix) => StorageMessage::ListKeysResponse(
                    data.keys()
                        .filter(|x| x.starts_with(&prefix))
                        .map(|x| x.into())
                        .collect(),
                ),
                m => panic!("unexcepted message {m:?}"),
            }));
        }
        HandleResult::Discard
    }
}
