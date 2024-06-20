use super::Result;
use crate::{ipc::StorageClient, StorageValue, UserAlarmBody};
pub struct UserAlarmStorage(pub StorageClient);

impl UserAlarmStorage {
    fn update_id_list(&self, list: Vec<usize>) -> Result<()> {
        self.0.set(
            "useralarm/list".into(),
            StorageValue::String(serde_json::to_string(&list).unwrap()),
        )
    }

    pub fn get_id_list(&self) -> Result<Vec<usize>> {
        Ok(self
            .0
            .get("useralarm/list".into())?
            .as_str()
            .map(|x| serde_json::from_str(&x).unwrap_or_default())
            .unwrap_or_default())
    }

    pub fn get(&self, id: usize) -> Result<UserAlarmBody> {
        match self.0.get(format!("useralarm/data/{id}"))? {
            StorageValue::String(x) => Ok(serde_json::from_str(&x).unwrap()),
            m => panic!("unexpected storage value {m:?}"),
        }
    }

    pub fn delete(&self, id: usize) -> Result<UserAlarmBody> {
        let bakup = self.get(id)?;
        // 删除元数据
        let list = self
            .get_id_list()?
            .into_iter()
            .filter(|x| *x != id)
            .collect();

        self.update_id_list(list)?;
        self.0
            .set(format!("useralarm/data/{id}"), StorageValue::None)?;
        Ok(bakup)
    }

    pub fn add(&self, body: UserAlarmBody) -> Result<usize> {
        // 申请一个id
        let mut id_list = self.get_id_list()?;
        let id = id_list.iter().map(|x| *x).max().unwrap_or(0) + 1;

        // 设置数据
        self.0.set(
            format!("useralarm/data/{id}"),
            StorageValue::String(serde_json::to_string(&body).unwrap()),
        )?;

        // 更新元数据
        id_list.push(id);
        self.update_id_list(id_list)?;
        Ok(id)
    }
}
