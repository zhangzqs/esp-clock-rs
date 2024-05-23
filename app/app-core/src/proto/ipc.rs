use std::{collections::HashSet, rc::Rc};

use crate::proto::{Context, Message, MessageTo, NodeName, TimeMessage};

use super::{
    HttpError, HttpMessage, HttpRequest, HttpResponse, NextSevenDaysWeather, PerformanceMessage,
    StorageError, StorageMessage, WeatherError, WeatherMessage,
};

type Callback<T> = Box<dyn FnOnce(T)>;
type ResultCallback<T, E> = Box<dyn FnOnce(Result<T, E>)>;

pub struct HttpClient(pub Rc<dyn Context>);

impl HttpClient {
    pub fn request(&self, request: HttpRequest, callback: ResultCallback<HttpResponse, HttpError>) {
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

pub struct TimestampClient(pub Rc<dyn Context>);

impl TimestampClient {
    pub fn get_timestamp_nanos(&self, callback: Callback<i128>) {
        self.0.async_call(
            NodeName::TimestampClient,
            Message::DateTime(TimeMessage::GetTimestampNanosRequest),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::DateTime(TimeMessage::GetTimestampNanosResponse(ts)) => ts,
                    m => panic!("unexpected response, {:?}", m),
                })
            }),
        )
    }
}

pub struct StorageClient(pub Rc<dyn Context>);

impl StorageClient {
    pub fn set(
        &self,
        key: String,
        value: Option<String>,
        callback: ResultCallback<(), StorageError>,
    ) {
        self.0.async_call(
            NodeName::Storage,
            Message::Storage(StorageMessage::SetRequest(key, value)),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Storage(StorageMessage::SetResponse) => Ok(()),
                    Message::Storage(StorageMessage::Error(e)) => Err(e),
                    m => panic!("unexpected message {:?}", m),
                });
            }),
        )
    }
    pub fn get(&self, key: String, callback: ResultCallback<Option<String>, StorageError>) {
        self.0.async_call(
            NodeName::Storage,
            Message::Storage(StorageMessage::GetRequest(key)),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Storage(StorageMessage::GetResponse(r)) => Ok(r),
                    Message::Storage(StorageMessage::Error(e)) => Err(e),
                    m => panic!("unexpected message {:?}", m),
                });
            }),
        );
    }

    pub fn list(&self, callback: ResultCallback<HashSet<String>, StorageError>) {
        self.0.async_call(
            NodeName::Storage,
            Message::Storage(StorageMessage::ListKeysRequest),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Storage(StorageMessage::ListKeysResponse(r)) => Ok(r),
                    Message::Storage(StorageMessage::Error(e)) => Err(e),
                    m => panic!("unexpected message {:?}", m),
                });
            }),
        );
    }
}

pub struct WeatherClient(pub Rc<dyn Context>);

impl WeatherClient {
    pub fn get_next_seven_days_weather(
        &self,
        callback: ResultCallback<NextSevenDaysWeather, WeatherError>,
    ) {
        self.0.async_call(
            NodeName::HttpClient,
            Message::Weather(WeatherMessage::GetNextSevenDaysWeatherRequest),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Weather(WeatherMessage::GetNextSevenDaysWeatherResponse(r)) => Ok(r),
                    Message::Weather(WeatherMessage::Error(e)) => Err(e),
                    m => panic!("unexpected message {:?}", m),
                });
            }),
        );
    }
}

pub struct PerformanceClient(pub Rc<dyn Context>);

impl PerformanceClient {
    pub fn get_free_heap_size(&self, callback: Callback<usize>) {
        self.0.async_call(
            NodeName::Performance,
            Message::Performance(PerformanceMessage::GetFreeHeapSizeRequest),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Performance(PerformanceMessage::GetFreeHeapSizeResponse(s)) => s,
                    m => panic!("unexpected response, {:?}", m),
                })
            }),
        )
    }

    pub fn get_largeest_free_block(&self, callback: Callback<usize>) {
        self.0.async_call(
            NodeName::Performance,
            Message::Performance(PerformanceMessage::GetLargestFreeBlock),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Performance(PerformanceMessage::GetLargestFreeBlockResponse(s)) => s,
                    m => panic!("unexpected response, {:?}", m),
                })
            }),
        )
    }

    pub fn get_fps(&self, callback: Callback<usize>) {
        self.0.async_call(
            NodeName::Performance,
            Message::Performance(PerformanceMessage::GetFpsRequest),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Performance(PerformanceMessage::GetFpsResponse(s)) => s,
                    m => panic!("unexpected response, {:?}", m),
                })
            }),
        )
    }
}
