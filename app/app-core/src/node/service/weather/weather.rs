use std::rc::Rc;

use crate::proto::*;
use serde::Deserialize;

use super::common::*;

#[derive(Deserialize, Debug, Clone)]
pub struct WeatherNowData {
    /// 温度，默认单位：摄氏度
    pub temp: Number<i8>,
    /// 天气图标代码
    pub icon: Number<u16>,
    /// 天气状况文字描述
    pub text: String,
    /// 相对湿度，百分比数值
    pub humidity: Number<u8>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WeatherNowOutput {
    pub code: ErrorCode,
    #[serde(rename = "updateTime")]
    pub update_time: Option<UtcDateTime>,
    pub now: Option<WeatherNowData>,
}

impl TryInto<proto::NowWeather> for WeatherNowOutput {
    type Error = WeatherError;

    fn try_into(self) -> Result<proto::NowWeather, Self::Error> {
        self.code.detect_error()?;
        let updated_time = self.update_time.ok_or(WeatherError::MissingFieldError(
            "missing field `updateTime`".into(),
        ))?;
        let now = self.now.ok_or(WeatherError::MissingFieldError(
            "missing field `now`".into(),
        ))?;
        Ok(NowWeather {
            updated_time: updated_time.into(),
            temp: now.temp.take(),
            icon: now.icon.take(),
            text: now.text,
            humidity: now.humidity.take(),
        })
    }
}

mod date_serde {
    use serde::Deserialize;
    use time::macros::format_description;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<time::Date, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let format = format_description!("[year]-[month]-[day]");
        let date = time::Date::parse(&s, &format).map_err(serde::de::Error::custom)?;
        Ok(date)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct WeatherForecastData {
    #[serde(rename = "fxDate", with = "date_serde")]
    pub fx_date: time::Date,
    /// 温度，默认单位：摄氏度
    #[serde(rename = "tempMax")]
    pub temp_max: Number<i8>,
    #[serde(rename = "tempMin")]
    pub temp_min: Number<i8>,
    /// 天气图标代码
    #[serde(rename = "iconDay")]
    pub icon_day: Number<u16>,
    #[serde(rename = "iconNight")]
    pub icon_night: Number<u16>,
    /// 天气状况文字描述
    #[serde(rename = "textDay")]
    pub text_day: String,
    #[serde(rename = "textNight")]
    pub text_night: String,
    /// 相对湿度，百分比数值
    pub humidity: Number<u8>,
}

impl From<WeatherForecastData> for proto::ForecastOneDayWeather {
    fn from(val: WeatherForecastData) -> Self {
        proto::ForecastOneDayWeather {
            date: val.fx_date,
            min_temp: val.temp_min.take(),
            max_temp: val.temp_max.take(),
            icon_day: val.icon_day.take(),
            text_day: val.text_day,
            icon_night: val.icon_night.take(),
            text_night: val.text_night,
            humidity: val.humidity.take(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct WeatherForecastOutput {
    pub code: ErrorCode,
    #[serde(rename = "updateTime")]
    pub update_time: Option<UtcDateTime>,
    pub daily: Option<Vec<WeatherForecastData>>,
}

impl TryInto<proto::ForecastWeather> for WeatherForecastOutput {
    type Error = WeatherError;

    fn try_into(self) -> Result<proto::ForecastWeather, Self::Error> {
        self.code.detect_error()?;
        let updated_time = self.update_time.ok_or(WeatherError::MissingFieldError(
            "missing field `updateTime`".into(),
        ))?;
        let daily = self.daily.ok_or(WeatherError::MissingFieldError(
            "missing field `daily`".into(),
        ))?;
        Ok(proto::ForecastWeather {
            updated_time: updated_time.into(),
            daily: daily.into_iter().map(Into::into).collect(),
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct AirQuality {
    #[serde(rename = "defaultLocalAqi")]
    pub default_local_aqi: bool,
    pub value: u16,
    pub category: String,
    pub color: RgbColor,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AirQualityNowOutput {
    pub code: ErrorCode,
    #[serde(rename = "updateTime")]
    pub update_time: Option<UtcDateTime>,
    pub aqi: Option<Vec<AirQuality>>,
}

impl TryInto<proto::NowAirQuality> for AirQualityNowOutput {
    type Error = WeatherError;

    fn try_into(self) -> Result<proto::NowAirQuality, Self::Error> {
        self.code.detect_error()?;
        let updated_time = self.update_time.ok_or(WeatherError::MissingFieldError(
            "missing field `updateTime`".into(),
        ))?;
        let aqi = self.aqi.ok_or(WeatherError::MissingFieldError(
            "missing field `aqi`".into(),
        ))?;
        let aq = aqi
            .into_iter()
            .filter(|x| x.default_local_aqi)
            .next()
            .ok_or(WeatherError::MissingFieldError(
                "missing defaultLocalAqi: true".into(),
            ))?;
        Ok(proto::NowAirQuality {
            updated_time: updated_time.into(),
            value: aq.value,
            category: aq.category,
            color: aq.color.into(),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct WeatherQueryInput {
    pub location: String,
    pub key: String,
}

impl WeatherQueryInput {
    pub fn request_forecast_weather(
        &self,
        ctx: Rc<dyn Context>,
        callback: Box<dyn FnOnce(Result<WeatherForecastOutput, WeatherError>)>,
    ) {
        ipc::HttpClient(ctx).request(
            HttpRequest {
                method: HttpRequestMethod::Get,
                url: format!(
                    "https://devapi.qweather.com/v7/weather/3d?gzip=n&lang=en&key={}&location={}",
                    self.key, self.location
                ),
            },
            Box::new(|r| {
                callback(match r {
                    Ok(x) => x
                        .body
                        .deserialize_by_json()
                        .map_err(|e| WeatherError::SerdeError(format!("{e}"))),
                    Err(e) => Err(WeatherError::HttpError(e)),
                });
            }),
        );
    }

    pub fn request_now_weather(
        &self,
        ctx: Rc<dyn Context>,
        callback: Box<dyn FnOnce(Result<WeatherNowOutput, WeatherError>)>,
    ) {
        ipc::HttpClient(ctx).request(
            HttpRequest {
                method: HttpRequestMethod::Get,
                url: format!(
                    "https://devapi.qweather.com/v7/weather/now?gzip=n&lang=en&key={}&location={}",
                    self.key, self.location
                ),
            },
            Box::new(|r| {
                callback(match r {
                    Ok(x) => x
                        .body
                        .deserialize_by_json()
                        .map_err(|e| WeatherError::SerdeError(format!("{e}"))),
                    Err(e) => Err(WeatherError::HttpError(e)),
                });
            }),
        );
    }

    pub fn request_now_air_quality(
        &self,
        ctx: Rc<dyn Context>,
        callback: Box<dyn FnOnce(Result<AirQualityNowOutput, WeatherError>)>,
    ) {
        ipc::HttpClient(ctx).request(
            HttpRequest {
                method: HttpRequestMethod::Get,
                url: format!(
                    "https://devapi.qweather.com/airquality/v1/now/{}?gzip=n&lang=en&key={}",
                    self.location, self.key
                ),
            },
            Box::new(|r| {
                callback(match r {
                    Ok(x) => x
                        .body
                        .deserialize_by_json()
                        .map_err(|e| WeatherError::SerdeError(format!("{e}"))),
                    Err(e) => Err(WeatherError::HttpError(e)),
                });
            }),
        );
    }
}
