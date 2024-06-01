use std::{fmt::Display, rc::Rc, str::FromStr};

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
    wea_img: String,
    tem: Number<f32>,
    tem1: Number<f32>,
    tem2: Number<f32>,
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
                m => unimplemented!("not supported {m}"),
            },
            air_quality_index: val.air.take(),
        }
    }
}

#[derive(Deserialize)]
struct YikeDaysWeatherResponse {
    #[serde(rename = "cityEn")]
    city: String,
    data: Vec<YikeOneDayWeather>,
}

impl From<YikeDaysWeatherResponse> for NextSevenDaysWeather {
    fn from(val: YikeDaysWeatherResponse) -> Self {
        NextSevenDaysWeather {
            city: val.city,
            data: val.data.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Deserialize)]
struct YikeNowWeatherResponse {
    #[serde(rename = "cityEn")]
    city: String,
    #[serde(flatten)]
    data: YikeOneDayWeather,
}

impl From<YikeNowWeatherResponse> for NowWeather {
    fn from(val: YikeNowWeatherResponse) -> Self {
        NowWeather {
            city: val.city,
            data: val.data.into(),
        }
    }
}

pub struct WeatherService {}

impl WeatherService {
    pub fn new() -> Self {
        Self {}
    }

    fn get_now_weather(seq: usize, ctx: Rc<dyn Context>) -> HandleResult {
        // 首次消息，进入pending状态
        let stg = ipc::StorageClient(ctx.clone());
        let appid: String = stg.get("weather/appid".into()).unwrap().into();
        let appsecret: String = stg.get("weather/appsecret".into()).unwrap().into();
        let http = HttpClient(ctx.clone());
        http.request(
            HttpRequest {
                method: HttpRequestMethod::Get,
                url: format!("http://v1.yiketianqi.com/api?unescape=1&version=v61&appid={appid}&appsecret={appsecret}"),
            },
            Box::new(move |r| {
                let x = match r {
                    Ok(x) => match x.body.deserialize_by_json::<YikeNowWeatherResponse>() {
                        Ok(x) => WeatherMessage::GetNowResponse(x.into()),
                        Err(e) => {
                            WeatherMessage::Error(WeatherError::SerdeError(e.to_string()))
                        }
                    },
                    Err(e) => WeatherMessage::Error(WeatherError::HttpError(e)),
                };
                ctx.async_ready(seq, Message::Weather(x));
            }),
        );
        HandleResult::Pending
    }

    fn get_next_seven_days_weather(seq: usize, ctx: Rc<dyn Context>) -> HandleResult {
        // 首次消息，进入pending状态
        let stg = ipc::StorageClient(ctx.clone());
        let appid: String = stg.get("weather/appid".into()).unwrap().into();
        let appsecret: String = stg.get("weather/appsecret".into()).unwrap().into();
        let http = HttpClient(ctx.clone());
        http.request(
            HttpRequest {
                method: HttpRequestMethod::Get,
                url: format!("http://v1.yiketianqi.com/api?unescape=1&version=v91&appid={appid}&appsecret={appsecret}"),
            },
            Box::new(move |r| {
                let x = match r {
                    Ok(x) => match x.body.deserialize_by_json::<YikeDaysWeatherResponse>() {
                        Ok(x) => WeatherMessage::GetNextSevenDaysWeatherResponse(x.into()),
                        Err(e) => {
                            WeatherMessage::Error(WeatherError::SerdeError(e.to_string()))
                        }
                    },
                    Err(e) => WeatherMessage::Error(WeatherError::HttpError(e)),
                };
                ctx.async_ready(seq, Message::Weather(x));
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
                WeatherMessage::GetNextSevenDaysWeatherRequest => {
                    return Self::get_next_seven_days_weather(seq, ctx);
                }
                WeatherMessage::GetNowRequest => {
                    return Self::get_now_weather(seq, ctx);
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
