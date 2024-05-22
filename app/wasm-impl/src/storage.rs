use std::rc::Rc;

use app_core::proto::{
    Context, HandleResult, Message, MessageTo, MessageWithHeader, Node, NodeName, StorageMessage,
};

pub struct LocalStorageService {}

impl Node for LocalStorageService {
    fn node_name(&self) -> NodeName {
        NodeName::Storage
    }

    fn handle_message(
        &mut self,
        _ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        if let Message::Storage(sm) = msg.body {
            let stg = web_sys::window().unwrap().local_storage().unwrap().unwrap();

            let resp = match sm {
                StorageMessage::GetRequest(key) => {
                    let ret = stg.get(&key).unwrap();
                    StorageMessage::GetResponse(ret)
                }

                StorageMessage::SetRequest(key, value) => {
                    if let Some(value) = value {
                        stg.set(&key, &value).unwrap();
                    } else {
                        stg.remove_item(&key).unwrap();
                    }
                    StorageMessage::SetResponse
                }

                StorageMessage::ListKeysRequest => {
                    let ret = stg
                        .get("meta")
                        .unwrap()
                        .map(|s| serde_json::from_str::<Vec<String>>(&s).unwrap())
                        .unwrap_or_default();
                    StorageMessage::ListKeysResponse(ret)
                }
                m => panic!("unexpected message {:?}", m),
            };
            return HandleResult::Finish(Message::Storage(resp));
        }
        HandleResult::Discard
    }
}
