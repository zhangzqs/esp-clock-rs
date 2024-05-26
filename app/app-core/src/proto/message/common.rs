use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Bytes(pub Vec<u8>);

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() < 64 {
            f.debug_tuple("Bytes").field(&self.0).finish()
        } else {
            f.debug_tuple("Bytes")
                .field(&format!("[u8;{}]", self.0.len()))
                .finish()
        }
    }
}
