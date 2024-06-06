use crate::StorageError;

use super::HttpError;
use serde::{Deserialize, Serialize};
use time::serde::rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NowWeather {
    /// 更新时间
    #[serde(with = "rfc3339")]
    pub updated_time: OffsetDateTime,
    /// 当前温度，摄氏度
    pub temp: i8,
    /// 天气图标
    pub icon: u16,
    /// 天气状况文字描述
    pub text: String,
    /// 相对湿度，百分比数值
    pub humidity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeatherError {
    StorageError(StorageError),
    SerdeError(String),
    HttpError(HttpError),
    ApiError(u16),
    MissingFieldError(String),
    MissingKey,
    MissingLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityLookUpItem {
    pub name: String,
    pub id: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastOneDayWeather {
    #[serde(with = "date_serde")]
    pub date: time::Date,
    pub min_temp: i8,
    pub max_temp: i8,
    pub icon_day: u16,
    pub text_day: String,
    pub icon_night: u16,
    pub text_night: String,
    pub humidity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastWeather {
    /// 更新时间
    #[serde(with = "rfc3339")]
    pub updated_time: OffsetDateTime,
    pub daily: Vec<ForecastOneDayWeather>,
}

mod date_serde {
    use serde::{Deserialize, Serializer};
    use time::macros::format_description;

    pub fn serialize<S>(t: &time::Date, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let fmt = format_description!("[year]-[month]-[day]");
        let s = t.format(fmt).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<time::Date, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let format = format_description!("[year]-[month]-[day]");
        let date = time::Date::parse(&s, &format).map_err(serde::de::Error::custom)?;
        Ok(date)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NowAirQuality {
    #[serde(with = "rfc3339")]
    pub updated_time: OffsetDateTime,
    pub value: u16,
    pub category: String,
    pub color: (u8, u8, u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub location_id: u32,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeatherMessage {
    Error(WeatherError),

    // 设置位置
    SetLocationRequest(Location),
    SetLocationResponse,

    // 获取位置
    GetLocationRequest,
    GetLocationResponse(Location),

    // 实时天气
    GetNowWeatherRequest,
    GetNowWeatherResponse(NowWeather),

    // 天气预报
    GetForecastWeatherRequest,
    GetForecastWeatherResponse(ForecastWeather),

    // 城市查询
    CityLookUpRequest(String),
    CityLookUpResponse(Vec<CityLookUpItem>),

    // 空气质量查询(location_id)
    GetNowAirQualityRequest,
    GetNowAirQualityResponse(NowAirQuality),
}
