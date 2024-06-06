use std::{fmt::Display, str::FromStr};

use crate::proto::*;
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Iso8601, OffsetDateTime};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Number<T>(T);

impl<'de, T: FromStr<Err = E>, E: Display> Deserialize<'de> for Number<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Ok(Self(
            String::deserialize(deserializer)?
                .parse::<T>()
                .map_err(serde::de::Error::custom)?,
        ))
    }
}

impl<T: FromStr + ToString> ToString for Number<T> {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl<T> Number<T> {
    pub fn take(self) -> T {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UtcDateTime(OffsetDateTime);

impl<'de> Deserialize<'de> for UtcDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let date = OffsetDateTime::parse(&s, &Iso8601::DATE_TIME_OFFSET)
            .map_err(serde::de::Error::custom)?;
        Ok(Self(date))
    }
}

impl From<UtcDateTime> for OffsetDateTime {
    fn from(val: UtcDateTime) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ErrorCode(Number<u16>);

impl ErrorCode {
    pub fn detect_error(self) -> Result<(), WeatherError> {
        match self.0.take() {
            200 | 206 => Ok(()),
            err_code => Err(WeatherError::ApiError(err_code)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct RgbColor(u8, u8, u8);

impl<'de> Deserialize<'de> for RgbColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut rgb_str = s.split(",");
        let r = rgb_str
            .next()
            .ok_or(serde::de::Error::custom("no red component"))?
            .parse()
            .map_err(serde::de::Error::custom)?;
        let g = rgb_str
            .next()
            .ok_or(serde::de::Error::custom("no green component"))?
            .parse()
            .map_err(serde::de::Error::custom)?;
        let b = rgb_str
            .next()
            .ok_or(serde::de::Error::custom("no blue component"))?
            .parse()
            .map_err(serde::de::Error::custom)?;
        Ok(Self(r, g, b))
    }
}

impl Into<(u8, u8, u8)> for RgbColor {
    fn into(self) -> (u8, u8, u8) {
        (self.0, self.1, self.2)
    }
}
