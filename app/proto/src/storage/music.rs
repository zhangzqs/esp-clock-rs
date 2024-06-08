use crate::{ipc::StorageClient, Bytes, StorageValue};

pub struct MusicStorage(pub StorageClient);
impl MusicStorage {
    fn update_list(&self, list: Vec<String>) {
        self.0
            .set(
                "music/list".into(),
                StorageValue::String(serde_json::to_string(&list).unwrap()),
            )
            .expect("update music list error");
    }

    pub fn get_list(&self) -> Vec<String> {
        self.0
            .get("music/list".into())
            .unwrap()
            .as_str()
            .map(|x| serde_json::from_str(&x).unwrap_or_default())
            .unwrap_or_default()
    }

    pub fn get_data(&self, filename: String) -> Vec<u8> {
        match self
            .0
            .get(format!("music/data/{filename}"))
            .expect("not found music data")
        {
            StorageValue::Bytes(bs) => bs.0,
            m => panic!("unexpected storage value {m:?}"),
        }
    }

    pub fn upload(&self, filename: String, data: Vec<u8>) {
        // 上传文件内容
        self.0
            .set(
                format!("music/data/{filename}"),
                StorageValue::Bytes(Bytes(data)),
            )
            .unwrap();
        // 更新元数据
        let mut list = self
            .get_list()
            .into_iter()
            .filter(|x| x != &filename) // 重复元素移除
            .collect::<Vec<_>>();
        list.push(filename);
        self.update_list(list);
    }
}
