use std::{collections::HashSet, rc::Rc};

use crate::{
    CityLookUpItem, Context, ForecastWeather, Location, Message, NodeName, NowAirQuality,
    NowWeather,
};

use super::{
    BuzzerMessage, Bytes, HttpError, HttpMessage, HttpRequest, HttpResponse, MidiError,
    MidiMessage, PerformanceMessage, StorageError, StorageMessage, StorageValue, ToneFrequency,
    ToneSeries, WeatherError, WeatherMessage,
};

type AsyncCallback<T> = Box<dyn FnOnce(T)>;

type AsyncResultCallback<T, E> = Box<dyn FnOnce(Result<T, E>)>;

#[derive(Clone)]
pub struct HttpClient(pub Rc<dyn Context>);

impl HttpClient {
    pub fn request(
        &self,
        request: HttpRequest,
        callback: AsyncResultCallback<HttpResponse, HttpError>,
    ) {
        self.0.async_call(
            NodeName::HttpClient,
            Message::Http(HttpMessage::Request(request)),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Http(HttpMessage::Response(resp)) => Ok(resp),
                    Message::Http(HttpMessage::Error(e)) => Err(e),
                    m => panic!("unexpected HandleResult {:?}", m),
                });
            }),
        )
    }
}
#[derive(Clone)]
pub struct StorageClient(pub Rc<dyn Context>);

impl StorageClient {
    pub fn set(&self, key: String, value: StorageValue) -> Result<(), StorageError> {
        let r = self.0.sync_call(
            NodeName::Storage,
            Message::Storage(StorageMessage::SetRequest(key, value)),
        );
        match r.unwrap() {
            Message::Storage(StorageMessage::SetResponse) => Ok(()),
            Message::Storage(StorageMessage::Error(e)) => Err(e),
            m => panic!("unexpected message {:?}", m),
        }
    }
    pub fn get(&self, key: String) -> Result<StorageValue, StorageError> {
        let r = self.0.sync_call(
            NodeName::Storage,
            Message::Storage(StorageMessage::GetRequest(key)),
        );
        match r.unwrap() {
            Message::Storage(StorageMessage::GetResponse(r)) => Ok(r),
            Message::Storage(StorageMessage::Error(e)) => Err(e),
            m => panic!("unexpected message {:?}", m),
        }
    }

    pub fn list(&self, prefix: String) -> Result<HashSet<String>, StorageError> {
        let r = self.0.sync_call(
            NodeName::Storage,
            Message::Storage(StorageMessage::ListKeysRequest(prefix)),
        );
        match r.unwrap() {
            Message::Storage(StorageMessage::ListKeysResponse(r)) => Ok(r),
            Message::Storage(StorageMessage::Error(e)) => Err(e),
            m => panic!("unexpected message {:?}", m),
        }
    }
}

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
            Message::Weather(WeatherMessage::SetLocationResponse) => {
                return Ok(());
            }
            Message::Weather(WeatherMessage::Error(e)) => {
                return Err(e);
            }
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
            Message::Weather(WeatherMessage::GetLocationResponse(r)) => {
                return Ok(r);
            }
            Message::Weather(WeatherMessage::Error(e)) => {
                return Err(e);
            }
            m => panic!("unexpected message {:?}", m),
        }
    }
}

#[derive(Clone)]
pub struct PerformanceClient(pub Rc<dyn Context>);

impl PerformanceClient {
    pub fn get_free_heap_size(&self) -> usize {
        let r = self.0.sync_call(
            NodeName::Performance,
            Message::Performance(PerformanceMessage::GetFreeHeapSizeRequest),
        );
        match r.unwrap() {
            Message::Performance(PerformanceMessage::GetFreeHeapSizeResponse(s)) => s,
            m => panic!("unexpected response, {:?}", m),
        }
    }

    pub fn get_largeest_free_block(&self) -> usize {
        let r = self.0.sync_call(
            NodeName::Performance,
            Message::Performance(PerformanceMessage::GetLargestFreeBlock),
        );
        match r.unwrap() {
            Message::Performance(PerformanceMessage::GetLargestFreeBlockResponse(s)) => s,
            m => panic!("unexpected response, {:?}", m),
        }
    }

    pub fn get_fps(&self) -> usize {
        let r = self.0.sync_call(
            NodeName::Performance,
            Message::Performance(PerformanceMessage::GetFpsRequest),
        );
        match r.unwrap() {
            Message::Performance(PerformanceMessage::GetFpsResponse(s)) => s,
            m => panic!("unexpected response, {:?}", m),
        }
    }
}

#[derive(Clone)]
pub struct MidiPlayerClient(pub Rc<dyn Context>);

impl MidiPlayerClient {
    pub fn play(&self, mid: Vec<u8>, callback: AsyncResultCallback<bool, MidiError>) {
        self.0.async_call(
            NodeName::Midi,
            Message::Midi(MidiMessage::PlayRequest(Bytes(mid))),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Midi(msg) => match msg {
                        MidiMessage::PlayResponse(is_finished) => Ok(is_finished),
                        MidiMessage::Error(e) => Err(e),
                        m => panic!("unexpected response, {:?}", m),
                    },
                    m => panic!("unexpected response, {:?}", m),
                });
            }),
        );
    }

    pub fn off(&self) {
        self.0
            .sync_call(NodeName::Midi, Message::Midi(MidiMessage::Off));
    }
}

#[derive(Clone)]
pub struct BuzzerClient(pub Rc<dyn Context>);

impl BuzzerClient {
    pub fn tone(&self, freq: ToneFrequency) {
        self.0.sync_call(
            NodeName::Buzzer,
            Message::Buzzer(BuzzerMessage::ToneForever(freq)),
        );
    }

    pub fn tone_series(&self, series: ToneSeries, callback: AsyncCallback<bool>) {
        self.0.async_call(
            NodeName::Buzzer,
            Message::Buzzer(BuzzerMessage::ToneSeriesRequest(series)),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Buzzer(BuzzerMessage::ToneSeriesResponse(is_finished)) => is_finished,
                    m => panic!("unexpected response, {:?}", m),
                })
            }),
        );
    }

    pub fn off(&self) {
        self.0
            .sync_call(NodeName::Buzzer, Message::Buzzer(BuzzerMessage::Off));
    }
}
