use crate::{ipc::StorageClient, Location, StorageValue, WeatherError};

type Result<T> = std::result::Result<T, WeatherError>;

pub struct WeatherStorage(pub StorageClient);

impl WeatherStorage {
    pub fn set_key(&self, key: String) -> Result<()> {
        self.0
            .set("weather/key".into(), StorageValue::String(key))
            .map_err(WeatherError::StorageError)?;
        Ok(())
    }

    pub fn set_location(&self, location_id: u32, location: String) -> Result<()> {
        self.0
            .set(
                "weather/location".into(),
                StorageValue::String(
                    serde_json::to_string(&Location {
                        location_id,
                        location,
                    })
                    .unwrap(),
                ),
            )
            .map_err(WeatherError::StorageError)?;
        Ok(())
    }
}
