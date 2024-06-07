type AsyncCallback<T> = Box<dyn FnOnce(T)>;
type AsyncResultCallback<T, E> = Box<dyn FnOnce(Result<T, E>)>;

mod buzzer;
mod httpclient;
mod midi;
mod performance;
mod storage;
mod weather;

pub use {
    buzzer::BuzzerClient, httpclient::HttpClient, midi::MidiPlayerClient,
    performance::PerformanceClient, storage::StorageClient, weather::WeatherClient,
};
