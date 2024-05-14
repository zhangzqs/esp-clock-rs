use crate::app::WeatherData;

#[derive(Debug, Clone)]
pub enum HomeMessage {
    RequestUpdateWeather,
    UpdateWeather(WeatherData),
}
