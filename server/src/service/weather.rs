use log::debug;
use poem::error::Result;
use poem_openapi::{param::Query, payload::Json, OpenApi};
use qweather_http_client::{
    ReqwestHttpAsyncClient, ReqwestHttpAsyncClientConfiguration, WEATHER_API_URL,
    WEATHER_DEV_API_URL,
};
use qweather_service::{CityLookUpInput, GeoApi, LocationInput, Weather};
use redis::AsyncCommands;
use serde::Deserialize;

use crate::error::ApiError;
use client::weather::*;

#[derive(Debug, Deserialize)]
pub struct WeatherServiceConfig {
    pub key: String,
    pub dev_api: bool,
}

pub struct WeatherService {
    client: ReqwestHttpAsyncClient,
    redis_cli: redis::Client,
}

impl WeatherService {
    pub fn new(config: WeatherServiceConfig, redis_cli: redis::Client) -> Self {
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
        Self { client, redis_cli }
    }
}

#[OpenApi]
impl WeatherService {
    /// 查找城市
    #[oai(path = "/weather/city_lookup", method = "get")]
    pub async fn city_lookup(&self, query: Query<String>) -> Result<Json<serde_json::Value>> {
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

    async fn now_remote(&self, city_id: String) -> Result<WeatherNowResponse, ApiError> {
        let weather = Weather::new(&self.client);

        let ret = weather
            .now(&qweather_service::WeatherInput {
                location: LocationInput::ID(city_id.clone()),
                ..Default::default()
            })
            .await
            .map_err(ApiError::from)?;
        if let Some(data) = ret.now {
            Ok(WeatherNowResponse {
                temp: data.temp.take(),
                humidity: data.humidity.take(),
                icon: data.icon,
                text: data.text,
            })
        } else {
            Err(ApiError::NotFound {
                resource: Some(format!("weather:now:{}", city_id)),
            })
        }
    }

    /// 查询实时天气
    #[oai(path = "/weather/now", method = "get")]
    pub async fn now(&self, city_id: Query<String>) -> Result<Json<serde_json::Value>> {
        let mut redis_conn = self
            .redis_cli
            .get_async_connection()
            .await
            .map_err(ApiError::from)?;
        let cache: Option<String> = redis_conn
            .get(&format!("weather:now:{}", &city_id.0))
            .await
            .map_err(ApiError::from)?;
        if let Some(data) = cache {
            debug!("cache hit: weather:now:{}", &city_id.0);
            return Ok(Json(serde_json::from_str(&data).map_err(ApiError::from)?));
        }

        debug!("cache miss: weather:now:{}", &city_id.0);
        let ret = self.now_remote(city_id.0.clone()).await?;
        debug!("cache set: weather:now:{} => {:?}", &city_id.0, &ret);
        redis_conn
            .set_ex(
                &format!("weather:now:{}", city_id.0),
                serde_json::to_string(&ret).map_err(ApiError::from)?,
                3600,
            )
            .await
            .map_err(ApiError::from)?;
        Ok(Json(serde_json::to_value(ret).map_err(ApiError::from)?))
    }
}
