pub type StorageError = String;

#[derive(Debug, Clone)]
pub enum StorageMessage {
    /// 错误定义
    Error(StorageError),

    /// 获取
    GetRequest(String),
    GetResponse(Option<String>),

    /// 设置，设置为None表示删除
    SetRequest(String, Option<String>),

    /// 列举出所有的keys
    ListKeysRequest,
    ListKeysResponse(Vec<String>),
}
