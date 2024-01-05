use poem::error::Result;
use poem_openapi::{param::Query, payload::Json, OpenApi};
use qweather_http_client::{
    ReqwestHttpAsyncClient, ReqwestHttpAsyncClientConfiguration, WEATHER_API_URL,
    WEATHER_DEV_API_URL,
};
use qweather_service::{CityLookUpInput, GeoApi, LocationInput, Weather};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct WeatherServiceConfig {
    pub key: String,
    pub dev_api: bool,
}

pub struct WeatherService {
    client: ReqwestHttpAsyncClient,
}

impl WeatherService {
    pub fn new(config: WeatherServiceConfig) -> Self {
        let client = ReqwestHttpAsyncClient::new(&ReqwestHttpAsyncClientConfiguration {
            key: config.key,
            weather_base_url: Some(
                if config.dev_api {
                    WEATHER_DEV_API_URL
                } else {
                    WEATHER_API_URL
                }
                .into(),
            ),
            ..Default::default()
        })
        .unwrap();
        Self { client }
    }
}

#[derive(Serialize)]
struct WeatherCityLookupItem {
    pub name: String,
    pub id: String,
}

#[derive(Serialize)]
struct WeatherCityLookupResponse {
    pub items: Vec<WeatherCityLookupItem>,
}

#[derive(Serialize)]
struct WeatherNowResponse {
    pub temp: i32,
    pub humidity: f32,
    pub icon: String,
    pub text: String,
}

#[OpenApi]
impl WeatherService {
    /// 查找城市
    #[oai(path = "/weather/city_lookup", method = "get")]
    async fn city_lookup(&self, query: Query<String>) -> Result<Json<serde_json::Value>> {
        let geo = GeoApi::new(&self.client);
        let query = CityLookUpInput {
            location: LocationInput::Text(query.0),
            ..Default::default()
        };
        let ret = geo.city_lookup(&query).await.map_err(ApiError::from)?;

        Ok(Json(
            serde_json::to_value(WeatherCityLookupResponse {
                items: ret
                    .location
                    .unwrap_or(vec![])
                    .into_iter()
                    .map(|x| WeatherCityLookupItem {
                        name: x.name,
                        id: x.id,
                    })
                    .collect(),
            })
            .map_err(ApiError::from)?,
        ))
    }

    /// 查询实时天气
    #[oai(path = "/weather/now", method = "get")]
    async fn now(&self, id: Query<String>) -> Result<Json<serde_json::Value>> {
        let weather = Weather::new(&self.client);

        let ret = weather
            .now(&qweather_service::WeatherInput {
                location: LocationInput::ID(id.0),
                ..Default::default()
            })
            .await
            .map_err(ApiError::from)?;
        println!("{:#?}", ret);
        if let Some(data) = ret.now {
            Ok(Json(
                serde_json::to_value(WeatherNowResponse {
                    temp: data.temp.take(),
                    humidity: data.humidity.take(),
                    icon: data.icon,
                    text: data.text,
                })
                .map_err(ApiError::from)?,
            ))
        } else {
            Err(ApiError {
                code: 404,
                msg: Some("location not found".to_string()),
            }
            .into())
        }
    }
}
