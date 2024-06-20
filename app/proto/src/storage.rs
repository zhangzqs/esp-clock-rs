mod music;
mod system;
mod useralarm;
mod weather;
mod wifi;

pub use {music::MusicStorage, system::SystemStorage, weather::WeatherStorage, wifi::WiFiStorage, useralarm::UserAlarmStorage};

use crate::StorageError;
pub(self) type Result<T> = std::result::Result<T, StorageError>;
