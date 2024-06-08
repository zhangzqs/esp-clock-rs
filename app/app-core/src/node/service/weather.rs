use std::{rc::Rc, time::Duration};

use time::OffsetDateTime;

use crate::proto::*;

mod common;
mod geo;
mod weather;

type Result<T> = std::result::Result<T, WeatherError>;

pub struct WeatherService {}

impl WeatherService {
    pub fn new() -> Self {
        Self {}
    }

    fn get_key(ctx: Rc<dyn Context>) -> Result<String> {
        ipc::StorageClient(ctx.clone())
            .get("weather/key".into())
            .map_err(WeatherError::StorageError)?
            .as_str()
            .ok_or(WeatherError::MissingKey)
    }

    fn get_location(ctx: Rc<dyn Context>) -> Result<Location> {
        let s = ipc::StorageClient(ctx.clone())
            .get("weather/location".into())
            .map_err(WeatherError::StorageError)?
            .as_str()
            .ok_or(WeatherError::MissingLocation)?;
        serde_json::from_str(&s).map_err(|e| WeatherError::SerdeError(format!("{e:?}")))
    }

    fn get_now_weather(seq: usize, ctx: Rc<dyn Context>) -> Result<HandleResult> {
        // 探测缓存
        if let Some(x) = ipc::StorageClient(ctx.clone())
            .get("weather/cache/now_weather".into())
            .map_err(WeatherError::StorageError)?
            .as_str()
        {
            match serde_json::from_str::<NowWeather>(&x) {
                Ok(x) => {
                    // https://dev.qweather.com/docs/best-practices/cache/ 缓存时间10min
                    if OffsetDateTime::now_utc() - x.updated_time <= Duration::from_secs(60 * 10) {
                        return Ok(HandleResult::Finish(Message::Weather(
                            WeatherMessage::GetNowWeatherResponse(x),
                        )));
                    }
                }
                Err(e) => {
                    return Ok(HandleResult::Finish(Message::Weather(
                        WeatherMessage::Error(WeatherError::SerdeError(format!(
                            "deserialize cache value err: {e:?}"
                        ))),
                    )));
                }
            };
        }

        // API获取数据
        weather::WeatherQueryInput {
            location: Self::get_location(ctx.clone())?.location_id.to_string(),
            key: Self::get_key(ctx.clone())?,
        }
        .request_now_weather(
            ctx.clone(),
            Box::new(move |r| {
                ctx.async_ready(
                    seq,
                    Message::Weather(match (move || r?.try_into())() {
                        Ok(x) => {
                            if let Err(e) = ipc::StorageClient(ctx.clone()).set(
                                "weather/cache/now_weather".into(),
                                StorageValue::String(serde_json::to_string(&x).unwrap()),
                            ) {
                                WeatherMessage::Error(WeatherError::StorageError(e))
                            } else {
                                WeatherMessage::GetNowWeatherResponse(x)
                            }
                        }
                        Err(e) => WeatherMessage::Error(e),
                    }),
                )
            }),
        );
        Ok(HandleResult::Pending)
    }

    fn get_forecast_weather(seq: usize, ctx: Rc<dyn Context>) -> Result<HandleResult> {
        // 探测缓存
        if let Some(x) = ipc::StorageClient(ctx.clone())
            .get("weather/cache/forecast_weather".into())
            .map_err(WeatherError::StorageError)?
            .as_str()
        {
            match serde_json::from_str::<ForecastWeather>(&x) {
                Ok(x) => {
                    // 缓存时间1h https://dev.qweather.com/docs/best-practices/cache/
                    if OffsetDateTime::now_utc() - x.updated_time <= Duration::from_secs(60 * 60) {
                        return Ok(HandleResult::Finish(Message::Weather(
                            WeatherMessage::GetForecastWeatherResponse(x),
                        )));
                    }
                }
                Err(e) => {
                    return Ok(HandleResult::Finish(Message::Weather(
                        WeatherMessage::Error(WeatherError::SerdeError(format!("{e:?}"))),
                    )));
                }
            };
        }

        weather::WeatherQueryInput {
            location: Self::get_location(ctx.clone())?.location_id.to_string(),
            key: Self::get_key(ctx.clone())?,
        }
        .request_forecast_weather(
            ctx.clone(),
            Box::new(move |r| {
                ctx.async_ready(
                    seq,
                    Message::Weather(match (move || r?.try_into())() {
                        Ok(x) => {
                            if let Err(e) = ipc::StorageClient(ctx.clone()).set(
                                "weather/cache/forecast_weather".into(),
                                StorageValue::String(serde_json::to_string(&x).unwrap()),
                            ) {
                                WeatherMessage::Error(WeatherError::StorageError(e))
                            } else {
                                WeatherMessage::GetForecastWeatherResponse(x)
                            }
                        }
                        Err(e) => WeatherMessage::Error(e),
                    }),
                )
            }),
        );
        Ok(HandleResult::Pending)
    }

    fn city_lookup(seq: usize, ctx: Rc<dyn Context>, location: String) -> Result<HandleResult> {
        geo::GeoCityLookupInput {
            location,
            key: Self::get_key(ctx.clone())?,
            number: Some(5),
        }
        .request(
            ctx.clone(),
            Box::new(move |r| {
                ctx.async_ready(
                    seq,
                    Message::Weather(match (move || r?.try_into())() {
                        Ok(x) => WeatherMessage::CityLookUpResponse(x),
                        Err(e) => WeatherMessage::Error(e),
                    }),
                );
            }),
        );
        Ok(HandleResult::Pending)
    }

    fn get_now_air_quality(seq: usize, ctx: Rc<dyn Context>) -> Result<HandleResult> {
        // 探测缓存
        if let Some(x) = ipc::StorageClient(ctx.clone())
            .get("weather/cache/now_air_quality".into())
            .map_err(WeatherError::StorageError)?
            .as_str()
        {
            match serde_json::from_str::<NowAirQuality>(&x) {
                Ok(x) => {
                    // https://dev.qweather.com/docs/best-practices/cache/ 缓存时间30min
                    if OffsetDateTime::now_utc() - x.updated_time <= Duration::from_secs(60 * 30) {
                        return Ok(HandleResult::Finish(Message::Weather(
                            WeatherMessage::GetNowAirQualityResponse(x),
                        )));
                    }
                }
                Err(e) => {
                    return Ok(HandleResult::Finish(Message::Weather(
                        WeatherMessage::Error(WeatherError::SerdeError(format!("{e:?}"))),
                    )));
                }
            };
        }
        weather::WeatherQueryInput {
            location: Self::get_location(ctx.clone())?.location_id.to_string(),
            key: Self::get_key(ctx.clone())?,
        }
        .request_now_air_quality(
            ctx.clone(),
            Box::new(move |r| {
                ctx.async_ready(
                    seq,
                    Message::Weather(match (move || r?.try_into())() {
                        Ok(x) => {
                            if let Err(e) = ipc::StorageClient(ctx.clone()).set(
                                "weather/cache/now_air_quality".into(),
                                StorageValue::String(serde_json::to_string(&x).unwrap()),
                            ) {
                                WeatherMessage::Error(WeatherError::StorageError(e))
                            } else {
                                WeatherMessage::GetNowAirQualityResponse(x)
                            }
                        }
                        Err(e) => WeatherMessage::Error(e),
                    }),
                )
            }),
        );
        Ok(HandleResult::Pending)
    }

    fn handle_error(r: Result<HandleResult>) -> HandleResult {
        match r {
            Ok(x) => x,
            Err(e) => HandleResult::Finish(Message::Weather(WeatherMessage::Error(e))),
        }
    }
}

impl Node for WeatherService {
    fn node_name(&self) -> NodeName {
        NodeName::WeatherClient
    }

    fn handle_message(
        &self,
        ctx: std::rc::Rc<dyn Context>,
        msg: MessageWithHeader,
    ) -> HandleResult {
        let seq = msg.seq;
        match msg.body {
            Message::Weather(msg) => match msg {
                WeatherMessage::CityLookUpRequest(q) => {
                    return Self::handle_error(Self::city_lookup(seq, ctx, q));
                }
                WeatherMessage::GetForecastWeatherRequest => {
                    return Self::handle_error(Self::get_forecast_weather(seq, ctx));
                }
                WeatherMessage::GetNowWeatherRequest => {
                    return Self::handle_error(Self::get_now_weather(seq, ctx));
                }
                WeatherMessage::GetNowAirQualityRequest => {
                    return Self::handle_error(Self::get_now_air_quality(seq, ctx));
                }
                WeatherMessage::GetLocationRequest => {
                    return Self::handle_error(Self::get_location(ctx).map(|x| {
                        HandleResult::Finish(Message::Weather(WeatherMessage::GetLocationResponse(
                            x,
                        )))
                    }));
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
