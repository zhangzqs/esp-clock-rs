use std::rc::Rc;

use crate::proto::{Context, Message, MessageTo, NodeName, TimeMessage};

use super::{
    HttpError, HttpMessage, HttpRequest, HttpResponse, NextSevenDaysWeather, StorageError,
    StorageMessage, WeatherError, WeatherMessage,
};

type Callback<T> = Box<dyn FnOnce(T)>;
type ResultCallback<T, E> = Box<dyn FnOnce(Result<T, E>)>;

pub struct HttpClient(pub Rc<dyn Context>);

impl HttpClient {
    pub fn request(&self, request: HttpRequest, callback: ResultCallback<HttpResponse, HttpError>) {
        self.0.send_message_with_reply_once(
            MessageTo::Point(NodeName::HttpClient),
            Message::Http(HttpMessage::Request(request)),
            Box::new(|_, r| {
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
        self.0.send_message_with_reply_once(
            MessageTo::Point(NodeName::TimestampClient),
            Message::DateTime(TimeMessage::GetTimestampNanosRequest),
            Box::new(|_, r| {
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
    pub fn set_storage(
        &self,
        key: String,
        value: Option<String>,
        callback: ResultCallback<(), StorageError>,
    ) {
        self.0.send_message_with_reply_once(
            MessageTo::Point(NodeName::Storage),
            Message::Storage(StorageMessage::SetRequest(key, value)),
            Box::new(|_, r| {}),
        )
    }
    pub fn get_storage(&self, key: String, callback: ResultCallback<Option<String>, StorageError>) {
        self.0.send_message_with_reply_once(
            MessageTo::Point(NodeName::Storage),
            Message::Storage(StorageMessage::GetRequest(key)),
            Box::new(|_, r| {}),
        );
    }

    pub fn list_keys(&self, callback: ResultCallback<(), StorageError>) {
        self.0.send_message_with_reply_once(
            MessageTo::Point(NodeName::Storage),
            Message::Storage(StorageMessage::ListKeysRequest),
            Box::new(|_, r| {}),
        );
    }
}

pub struct WeatherClient(pub Rc<dyn Context>);

impl WeatherClient {
    pub fn get_next_seven_days_weather(
        &self,
        callback: ResultCallback<NextSevenDaysWeather, WeatherError>,
    ) {
        self.0.send_message_with_reply_once(
            MessageTo::Point(NodeName::HttpClient),
            Message::Weather(WeatherMessage::GetNextSevenDaysWeatherRequest),
            Box::new(|_, r| {
                callback(match r.unwrap() {
                    Message::Weather(WeatherMessage::GetNextSevenDaysWeatherResponse(r)) => Ok(r),
                    Message::Weather(WeatherMessage::Error(e)) => Err(e),
                    m => panic!("unexpected message {:?}", m),
                });
            }),
        );
    }
}
