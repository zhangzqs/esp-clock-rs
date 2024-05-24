use std::{fmt::Display, str::FromStr};

use crate::proto::*;
use serde::{Deserialize, Serialize};
use time::macros::format_description;

use self::ipc::HttpClient;

#[derive(Debug, Clone, Copy, Serialize)]
struct Number<T>(T);

impl<'de, T: FromStr<Err = E>, E: Display> Deserialize<'de> for Number<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Ok(Self(
            String::deserialize(deserializer)?
                .parse::<T>()
                .map_err(serde::de::Error::custom)?,
        ))
    }
}

impl<T: FromStr + ToString> ToString for Number<T> {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl<T> Number<T> {
    pub fn take(self) -> T {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
struct Date(time::Date);

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let format = format_description!("[year]-[month]-[day]");
        let date = time::Date::parse(&s, &format).map_err(serde::de::Error::custom)?;
        Ok(Self(date))
    }
}

impl Date {
    pub fn take(self) -> time::Date {
        self.0
    }
}

#[derive(Deserialize)]
struct YikeOneDayWeather {
    date: Date,
    wea: String,
    wea_img: String,
    tem: Number<i8>,
    tem1: Number<i8>,
    tem2: Number<i8>,
    humidity: String,
    air: Number<u16>,
}

impl From<YikeOneDayWeather> for OneDayWeather {
    fn from(val: YikeOneDayWeather) -> Self {
        OneDayWeather {
            date: val.date.take(),
            now_temperature: val.tem.take(),
            max_temperature: val.tem1.take(),
            min_temperature: val.tem2.take(),
            humidity: val.humidity.strip_suffix('%').unwrap().parse().unwrap(),
            state: match val.wea_img.as_str() {
                "xue" => WeatherState::Snow,
                "lei" => WeatherState::Thunder,
                "shachen" => WeatherState::Sandstorm,
                "wu" => WeatherState::Fog,
                "bingbao" => WeatherState::Hail,
                "yun" => WeatherState::Cloudy,
                "yu" => WeatherState::Rain,
                "yin" => WeatherState::Overcast,
                "qing" => WeatherState::Sunny,
                _ => todo!("not supported {}", val.wea),
            },
            state_description: val.wea,
            air_quality_index: val.air.take(),
        }
    }
}

#[derive(Deserialize)]
struct YikeWeatherResponse {
    city: String,
    data: Vec<YikeOneDayWeather>,
}

impl From<YikeWeatherResponse> for NextSevenDaysWeather {
    fn from(val: YikeWeatherResponse) -> Self {
        NextSevenDaysWeather {
            city: val.city,
            data: val.data.into_iter().map(Into::into).collect(),
        }
    }
}

pub struct WeatherService {}

impl WeatherService {
    pub fn new() -> Self {
        Self {}
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
        match msg.body {
            Message::Weather(WeatherMessage::GetNextSevenDaysWeatherRequest) => {
                // 首次消息，进入pending状态
                let stg = ipc::StorageClient(ctx.clone());
                let appid = stg.get("weather/appid".into()).unwrap().unwrap_or_default();
                let appsecret = stg
                    .get("weather/appsecret".into())
                    .unwrap()
                    .unwrap_or_default();
                HttpClient(ctx.clone()).request(
                    HttpRequest {
                        method: HttpRequestMethod::Get,
                        url: format!("http://v1.yiketianqi.com/api?unescape=1&version=v91&appid={appid}&appsecret={appsecret}"),
                    },
                    Box::new(move |r| {
                        let x = match r {
                            Ok(x) => match x.body.deserialize_by_json::<YikeWeatherResponse>() {
                                Ok(x) => WeatherMessage::GetNextSevenDaysWeatherResponse(x.into()),
                                Err(e) => {
                                    WeatherMessage::Error(WeatherError::SerdeError(e.to_string()))
                                }
                            },
                            Err(e) => WeatherMessage::Error(WeatherError::HttpError(e)),
                        };
                        ctx.async_ready(msg.seq, Message::Weather(x));
                    }),
                );
                return HandleResult::Pending;
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
