use std::fmt;

use base64::{prelude::BASE64_STANDARD, Engine};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Bytes(pub Vec<u8>);

impl From<Bytes> for Vec<u8> {
    fn from(val: Bytes) -> Self {
        val.0
    }
}

impl Bytes {
    pub fn to_base64(&self) -> String {
        let mut s = String::new();
        BASE64_STANDARD.encode_string(&self.0, &mut s);
        s
    }
}

impl Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = String::new();
        BASE64_STANDARD.encode_string(&self.0, &mut s);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let mut v = Vec::new();
        BASE64_STANDARD
            .decode_vec(String::deserialize(deserializer)?, &mut v)
            .map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

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

pub type Rgb888Color = (u8, u8, u8);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ImageColorMode {
    BinaryColor { on: Rgb888Color, off: Rgb888Color },
    Rgb565,
    Rgb888,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Image {
    pub width: u16,
    pub height: u16,
    pub color_mode: ImageColorMode,
    pub data: Bytes,
}
