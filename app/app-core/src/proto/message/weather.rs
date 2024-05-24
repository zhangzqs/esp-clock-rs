use super::HttpError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeatherState {
    /// 雪
    Snow,
    /// 雷
    Thunder,
    /// 沙尘暴
    Sandstorm,
    /// 雾天
    Fog,
    /// 冰雹
    Hail,
    /// 多云
    Cloudy,
    /// 下雨
    Rain,
    /// 阴天
    Overcast,
    /// 晴天
    Sunny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AirLevel {
    /// 优
    Good,
    /// 良
    Moderate,
    /// 轻度污染
    UnhealthyForSensitiveGroups,
    /// 中度污染
    Unhealthy,
    /// 重度污染
    VeryUnhealthy,
    /// 严重污染
    Hazardous,
}

mod date_serde {
    use serde::{Deserialize, Serializer};
    use time::macros::{format_description};

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
pub struct OneDayWeather {
    #[serde(with = "date_serde")]
    pub date: time::Date,
    pub now_temperature: i8,
    pub max_temperature: i8,
    pub min_temperature: i8,
    pub humidity: i8,
    pub state: WeatherState,
    pub state_description: String,
    pub air_quality_index: u16,
}

impl OneDayWeather {
    pub fn get_air_level(&self) -> AirLevel {
        match self.air_quality_index {
            0..=50 => AirLevel::Good,
            51..=100 => AirLevel::Moderate,
            101..=150 => AirLevel::UnhealthyForSensitiveGroups,
            151..=200 => AirLevel::Unhealthy,
            201..=300 => AirLevel::VeryUnhealthy,
            _ => AirLevel::Hazardous,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextSevenDaysWeather {
    pub city: String,
    pub data: Vec<OneDayWeather>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeatherError {
    SerdeError(String),
    HttpError(HttpError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeatherMessage {
    Error(WeatherError),
    GetNextSevenDaysWeatherRequest,
    GetNextSevenDaysWeatherResponse(NextSevenDaysWeather),
}
