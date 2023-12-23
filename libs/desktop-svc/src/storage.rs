use std::error::Error;

use embedded_svc::storage::{RawStorage, StorageBase};

pub struct KVStorage {
    db: sled::Db,
}

impl KVStorage {
    pub fn new<P>(path: P) -> Result<Self, KVStorageError>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(Self {
            db: sled::open(path)?,
        })
    }
}

#[derive(Debug)]
pub enum KVStorageError {
    SledError(sled::Error),
    Other(Box<dyn Error>),
}

impl From<sled::Error> for KVStorageError {
    fn from(error: sled::Error) -> Self {
        Self::SledError(error)
    }
}

impl StorageBase for KVStorage {
    type Error = KVStorageError;

    fn contains(&self, name: &str) -> Result<bool, Self::Error> {
        Ok(self.db.contains_key(name)?)
    }

    fn remove(&mut self, name: &str) -> Result<bool, Self::Error> {
        let ret = self.db.remove(name)?;
        Ok(ret.is_some())
    }
}

impl RawStorage for KVStorage {
    fn len(&self, name: &str) -> Result<Option<usize>, Self::Error> {
        let ret = self.db.get(name)?;
        Ok(ret.map(|v| v.len()))
    }

    fn get_raw<'a>(&self, name: &str, buf: &'a mut [u8]) -> Result<Option<&'a [u8]>, Self::Error> {
        let ret = self.db.get(name)?;
        if let Some(v) = ret {
            let len = std::cmp::min(v.len(), buf.len());
            buf[..len].copy_from_slice(&v[..len]);
            Ok(Some(&buf[..len]))
        } else {
            Ok(None)
        }
    }

    fn set_raw(&mut self, name: &str, buf: &[u8]) -> Result<bool, Self::Error> {
        let ret = self.db.insert(name, buf)?;
        Ok(ret.is_some())
    }
}
