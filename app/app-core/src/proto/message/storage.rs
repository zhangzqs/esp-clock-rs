use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StorageError {
    Other(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StorageMessage {
    /// 错误定义
    Error(StorageError),

    /// 获取
    GetRequest(String),
    GetResponse(Option<String>),

    /// 设置，设置为None表示删除
    SetRequest(String, Option<String>),
    SetResponse,

    /// 列举出所有的keys
    ListKeysRequest,
    ListKeysResponse(HashSet<String>),
}
