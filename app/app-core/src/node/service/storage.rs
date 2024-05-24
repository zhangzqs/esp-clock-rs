use std::{cell::RefCell, collections::HashMap};

use crate::proto::*;

pub struct MockStorageService {
    data: RefCell<HashMap<String, String>>,
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
                    StorageMessage::GetResponse(data.get(&k).map(|x| x.into()))
                }
                StorageMessage::SetRequest(k, v) => {
                    if let Some(v) = v {
                        data.insert(k, v);
                    } else {
                        data.remove(&k);
                    }
                    StorageMessage::SetResponse
                }
                StorageMessage::ListKeysRequest => {
                    StorageMessage::ListKeysResponse(data.keys().map(|x| x.into()).collect())
                }
                m => panic!("unexcepted message {m:?}"),
            }));
        }
        HandleResult::Discard
    }
}
