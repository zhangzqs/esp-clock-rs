use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WeatherCityLookupItem {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WeatherCityLookupResponse {
    pub items: Vec<WeatherCityLookupItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WeatherNowResponse {
    pub temp: i32,
    pub humidity: f32,
    pub icon: String,
    pub text: String,
}
