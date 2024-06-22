mod music;
mod system;
mod useralarm;
mod weather;
mod wifi;

pub use {
    music::MusicStorage, system::SystemStorage, useralarm::UserAlarmStorage,
    weather::WeatherStorage, wifi::WiFiStorage,
};

use crate::StorageError;
type Result<T> = std::result::Result<T, StorageError>;
