use std::rc::Rc;

use crate::{
    CityLookUpItem, Context, ForecastWeather, Location, Message, NodeName, NowAirQuality,
    NowWeather,
};

use crate::message::{WeatherError, WeatherMessage};

use super::AsyncResultCallback;

#[derive(Clone)]
pub struct WeatherClient(pub Rc<dyn Context>);

impl WeatherClient {
    pub fn city_lookup(
        &self,
        query: String,
        callback: AsyncResultCallback<Vec<CityLookUpItem>, WeatherError>,
    ) {
        self.0.async_call(
            NodeName::WeatherClient,
            Message::Weather(WeatherMessage::CityLookUpRequest(query)),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Weather(WeatherMessage::CityLookUpResponse(r)) => Ok(r),
                    Message::Weather(WeatherMessage::Error(e)) => Err(e),
                    m => panic!("unexpected message {:?}", m),
                })
            }),
        );
    }

    pub fn get_forecast_weather(
        &self,
        callback: AsyncResultCallback<ForecastWeather, WeatherError>,
    ) {
        self.0.async_call(
            NodeName::WeatherClient,
            Message::Weather(WeatherMessage::GetForecastWeatherRequest),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Weather(WeatherMessage::GetForecastWeatherResponse(r)) => Ok(r),
                    Message::Weather(WeatherMessage::Error(e)) => Err(e),
                    m => panic!("unexpected message {:?}", m),
                });
            }),
        );
    }

    pub fn get_now_weather(&self, callback: AsyncResultCallback<NowWeather, WeatherError>) {
        self.0.async_call(
            NodeName::WeatherClient,
            Message::Weather(WeatherMessage::GetNowWeatherRequest),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Weather(WeatherMessage::GetNowWeatherResponse(r)) => Ok(r),
                    Message::Weather(WeatherMessage::Error(e)) => Err(e),
                    m => panic!("unexpected message {:?}", m),
                });
            }),
        );
    }

    pub fn get_now_air_quality(&self, callback: AsyncResultCallback<NowAirQuality, WeatherError>) {
        self.0.async_call(
            NodeName::WeatherClient,
            Message::Weather(WeatherMessage::GetNowAirQualityRequest),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Weather(WeatherMessage::GetNowAirQualityResponse(r)) => Ok(r),
                    Message::Weather(WeatherMessage::Error(e)) => Err(e),
                    m => panic!("unexpected message {:?}", m),
                });
            }),
        );
    }

    pub fn set_location(&self, loc: Location) -> Result<(), WeatherError> {
        match self
            .0
            .sync_call(
                NodeName::WeatherClient,
                Message::Weather(WeatherMessage::SetLocationRequest(loc)),
            )
            .unwrap()
        {
            Message::Weather(WeatherMessage::SetLocationResponse) => Ok(()),
            Message::Weather(WeatherMessage::Error(e)) => Err(e),
            m => panic!("unexpected message {:?}", m),
        }
    }

    pub fn get_location(&self) -> Result<Location, WeatherError> {
        match self
            .0
            .sync_call(
                NodeName::WeatherClient,
                Message::Weather(WeatherMessage::GetLocationRequest),
            )
            .unwrap()
        {
            Message::Weather(WeatherMessage::GetLocationResponse(r)) => Ok(r),
            Message::Weather(WeatherMessage::Error(e)) => Err(e),
            m => panic!("unexpected message {:?}", m),
        }
    }
}
