use std::rc::Rc;

use crate::proto::*;

mod common;
mod geo;
mod weather;

pub struct WeatherService {}

impl WeatherService {
    pub fn new() -> Self {
        Self {}
    }

    fn get_key(ctx: Rc<dyn Context>) -> String {
        ipc::StorageClient(ctx.clone())
            .get("weather/key".into())
            .expect("no weather key")
            .into()
    }
    fn get_now_weather(seq: usize, ctx: Rc<dyn Context>, q: WeatherQuery) -> HandleResult {
        // 首次消息，进入pending状态
        let key = Self::get_key(ctx.clone());
        weather::WeatherQueryInput {
            location: match q {
                WeatherQuery::LocationID(id) => id.to_string(),
                WeatherQuery::LatLong(lat, long) => format!("{lat},{long}"),
            },
            key,
        }
        .request_now(
            ctx.clone(),
            Box::new(move |r| {
                ctx.async_ready(
                    seq,
                    Message::Weather(match (move || r?.try_into())() {
                        Ok(x) => WeatherMessage::GetNowResponse(x),
                        Err(e) => WeatherMessage::Error(e),
                    }),
                )
            }),
        );
        HandleResult::Pending
    }

    fn get_forecast_weather(
        seq: usize,
        ctx: Rc<dyn Context>,
        q: WeatherQuery,
        ds: WeatherForecastDays,
    ) -> HandleResult {
        // 首次消息，进入pending状态
        let key = Self::get_key(ctx.clone());
        weather::WeatherQueryInput {
            location: match q {
                WeatherQuery::LocationID(id) => id.to_string(),
                WeatherQuery::LatLong(lat, long) => format!("{lat},{long}"),
            },
            key,
        }
        .request_forecast(
            ctx.clone(),
            ds,
            Box::new(move |r| {
                ctx.async_ready(
                    seq,
                    Message::Weather(match (move || r?.try_into())() {
                        Ok(x) => WeatherMessage::GetForecastWeatherResponse(x),
                        Err(e) => WeatherMessage::Error(e),
                    }),
                )
            }),
        );
        HandleResult::Pending
    }

    fn city_lookup(seq: usize, ctx: Rc<dyn Context>, location: String) -> HandleResult {
        let key = Self::get_key(ctx.clone());
        geo::GeoCityLookupInput {
            location,
            key,
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
        HandleResult::Pending
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
                    return Self::city_lookup(seq, ctx, q);
                }
                WeatherMessage::GetForecastWeatherRequest(q, ds) => {
                    return Self::get_forecast_weather(seq, ctx, q, ds);
                }
                WeatherMessage::GetNowRequest(q) => {
                    return Self::get_now_weather(seq, ctx, q);
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
