#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct OneDayWeather {
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

#[derive(Debug, Clone)]
pub struct NextSevenDaysWeather {
    pub city: String,
    pub data: Vec<OneDayWeather>,
}

#[derive(Debug, Clone)]
pub enum WeatherMessage {
    GetNextSevenDaysWeatherRequest,
    GetNextSevenDaysWeatherResponse(NextSevenDaysWeather),
}
