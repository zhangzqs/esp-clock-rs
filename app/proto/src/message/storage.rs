use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use time::OffsetDateTime;

use super::Bytes;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StorageError {
    IOError(String),
    TypeError(String),
    Other(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StorageValue {
    None,
    Bytes(Bytes),
    String(String),
}

impl StorageValue {
    pub fn as_str<E>(self, e: E) -> Result<String, E> {
        match self {
            Self::String(x) => Ok(x),
            _ => Err(e),
        }
    }
}

impl From<StorageValue> for String {
    fn from(val: StorageValue) -> Self {
        match val {
            StorageValue::String(x) => x,
            m => panic!("type unmatch err: {m:?}"),
        }
    }
}

impl From<StorageValue> for Bytes {
    fn from(val: StorageValue) -> Self {
        match val {
            StorageValue::Bytes(x) => x,
            m => panic!("type unmatch err: {m:?}"),
        }
    }
}

impl From<String> for StorageValue {
    fn from(value: String) -> Self {
        StorageValue::String(value)
    }
}

impl From<OffsetDateTime> for StorageValue {
    fn from(value: OffsetDateTime) -> Self {
        StorageValue::String(value.to_string())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StorageMessage {
    /// 错误定义
    Error(StorageError),

    /// 获取
    GetRequest(String),
    GetResponse(StorageValue),

    /// 设置，设置为None表示删除
    SetRequest(String, StorageValue),
    SetResponse,

    /// 根据给定一个前缀，列举出所有的keys
    ListKeysRequest(String),
    ListKeysResponse(HashSet<String>),
}
