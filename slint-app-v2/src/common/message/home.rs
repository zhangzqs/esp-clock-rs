use crate::ui::WeatherData;

#[derive(Debug, Clone)]
pub enum HomeMessage {
    RequestUpdateWeather,
    UpdateWeather(WeatherData),
}
