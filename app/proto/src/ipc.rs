type AsyncCallback<T> = Box<dyn FnOnce(T)>;
type AsyncResultCallback<T, E> = Box<dyn FnOnce(Result<T, E>)>;

mod buzzer;
mod httpclient;
mod midi;
mod notifaction;
mod storage;
mod system;
mod useralarm;
mod weather;

pub use {
    buzzer::BuzzerClient, httpclient::HttpClient, midi::MidiPlayerClient,
    notifaction::NotifactionClient, storage::StorageClient, system::SystemClient,
    useralarm::UserAlarmClient, weather::WeatherClient,
};
