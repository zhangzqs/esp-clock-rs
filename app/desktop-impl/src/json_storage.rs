use std::{cell::RefCell, collections::HashMap, io::Seek};

use app_core::proto::*;

pub struct JsonStorageService {
    json_file: RefCell<std::fs::File>,
    data: RefCell<HashMap<String, StorageValue>>,
}

impl JsonStorageService {
    pub fn new(json_file_path: &str) -> Self {
        let json_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(json_file_path)
            .unwrap();
        let data: HashMap<String, StorageValue> =
            serde_json::from_reader(&json_file).expect("文件读取失败");
        log::debug!("配置加载成功：{:?}", data);
        Self {
            json_file: RefCell::new(json_file),
            data: RefCell::new(data),
        }
    }
}

impl Node for JsonStorageService {
    fn priority(&self) -> usize {
        9999
    }
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
                    self.json_file.borrow_mut().set_len(0).unwrap();
                    self.json_file.borrow_mut().rewind().unwrap();
                    serde_json::to_writer_pretty(&*self.json_file.borrow(), &*data).unwrap();
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
