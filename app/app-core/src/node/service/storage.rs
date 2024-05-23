use std::collections::HashMap;

use crate::proto::*;

pub struct MockStorageService {
    data: HashMap<String, String>,
}

impl MockStorageService {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Node for MockStorageService {
    fn node_name(&self) -> NodeName {
        NodeName::Storage
    }

    fn handle_message(
        &mut self,
        _ctx: std::rc::Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        if let Message::Storage(sm) = msg.body {
            return HandleResult::Finish(Message::Storage(match sm {
                StorageMessage::GetRequest(k) => {
                    StorageMessage::GetResponse(self.data.get(&k).map(|x| x.into()))
                }
                StorageMessage::SetRequest(k, v) => {
                    if let Some(v) = v {
                        self.data.insert(k, v);
                    } else {
                        self.data.remove(&k);
                    }
                    StorageMessage::SetResponse
                }
                StorageMessage::ListKeysRequest => {
                    StorageMessage::ListKeysResponse(self.data.keys().map(|x| x.into()).collect())
                }
                m => panic!("unexcepted message {m:?}"),
            }));
        }
        HandleResult::Discard
    }
}
