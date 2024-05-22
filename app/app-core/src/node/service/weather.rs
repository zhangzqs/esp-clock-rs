use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
    str::FromStr,
    sync::Arc,
};

use crate::proto::*;
use serde::{Deserialize, Serialize};
use time_macros::format_description;

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

impl Into<OneDayWeather> for YikeOneDayWeather {
    fn into(self) -> OneDayWeather {
        OneDayWeather {
            date: self.date.take(),
            now_temperature: self.tem.take(),
            max_temperature: self.tem1.take(),
            min_temperature: self.tem2.take(),
            humidity: self.humidity.strip_suffix("%").unwrap().parse().unwrap(),
            state: match self.wea_img.as_str() {
                "xue" => WeatherState::Snow,
                "lei" => WeatherState::Thunder,
                "shachen" => WeatherState::Sandstorm,
                "wu" => WeatherState::Fog,
                "bingbao" => WeatherState::Hail,
                "yun" => WeatherState::Cloudy,
                "yu" => WeatherState::Rain,
                "yin" => WeatherState::Overcast,
                "qing" => WeatherState::Sunny,
                _ => todo!("not supported {}", self.wea),
            },
            state_description: self.wea,
            air_quality_index: self.air.take(),
        }
    }
}

#[derive(Deserialize)]
struct YikeWeatherResponse {
    city: String,
    data: Vec<YikeOneDayWeather>,
}

impl Into<NextSevenDaysWeather> for YikeWeatherResponse {
    fn into(self) -> NextSevenDaysWeather {
        NextSevenDaysWeather {
            city: self.city,
            data: self.data.into_iter().map(Into::into).collect(),
        }
    }
}

pub struct WeatherClient {
    ready_resp: Rc<RefCell<HashMap<u32, NextSevenDaysWeather>>>,
}

impl WeatherClient {
    pub fn new() -> Self {
        Self {
            ready_resp: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl Node for WeatherClient {
    fn node_name(&self) -> NodeName {
        NodeName::WeatherClient
    }

    fn handle_message(
        &mut self,
        ctx: std::rc::Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        match msg.body {
            Message::Weather(WeatherMessage::GetNextSevenDaysWeatherRequest) => {
                // 出结果了
                if self.ready_resp.borrow().contains_key(&msg.seq) {
                    return HandleResult::Successful(Message::Weather(
                        WeatherMessage::GetNextSevenDaysWeatherResponse(
                            self.ready_resp.borrow_mut().remove(&msg.seq).unwrap(),
                        ),
                    ));
                }

                // 仍然需要pending
                if msg.is_pending {
                    return HandleResult::Pending;
                }

                // 首次消息，进入pending状态
                let ready_resp = self.ready_resp.clone();
                ctx.send_message_with_reply_once(
                    MessageTo::Point(NodeName::HttpClient),
                    Message::Http(HttpMessage::Request(Arc::new(HttpRequest {
                        method: HttpRequestMethod::Get,
                        url:
                            "http://v1.yiketianqi.com/api?unescape=1&version=v91&appid=&appsecret="
                                .into(),
                    }))),
                    Box::new(move |_, r| match r {
                        HandleResult::Successful(Message::Http(HttpMessage::Response(resp))) => {
                            if let HttpBody::Bytes(bs) = &resp.body {
                                let resp =
                                    serde_json::from_slice::<YikeWeatherResponse>(bs).unwrap();
                                ready_resp.borrow_mut().insert(msg.seq, resp.into());
                            }
                        }
                        _ => {}
                    }),
                );
                return HandleResult::Pending;
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
