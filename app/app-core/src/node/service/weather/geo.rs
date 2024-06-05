use std::rc::Rc;

use crate::proto::*;
use ipc::HttpClient;
use serde::Deserialize;

use super::common::*;

#[derive(Deserialize, Debug, Clone)]
pub struct GeoCityLookupInput {
    pub location: String,
    pub key: String,
    pub number: Option<u8>,
}

impl GeoCityLookupInput {
    fn to_url(&self) -> String {
        format!(
            "https://geoapi.qweather.com/v2/city/lookup?gzip=n&lang=en&key={}&location={}&number={}",
            self.key,
            self.location,
            self.number.unwrap_or(5),
        )
    }

    pub fn request(
        &self,
        ctx: Rc<dyn Context>,
        callback: Box<dyn FnOnce(Result<GeoCityLookupOutput, WeatherError>)>,
    ) {
        HttpClient(ctx).request(
            HttpRequest {
                method: HttpRequestMethod::Get,
                url: self.to_url(),
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

#[derive(Deserialize, Debug, Clone)]
pub struct GeoCityLookupItem {
    pub name: String,
    pub id: String,
    pub country: String,
}

impl From<GeoCityLookupItem> for proto::CityLookUpItem {
    fn from(val: GeoCityLookupItem) -> Self {
        proto::CityLookUpItem {
            name: val.name,
            id: val.id,
            country: val.country,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct GeoCityLookupOutput {
    pub code: ErrorCode,
    pub location: Vec<GeoCityLookupItem>,
}

impl TryInto<Vec<proto::CityLookUpItem>> for GeoCityLookupOutput {
    type Error = WeatherError;

    fn try_into(self) -> Result<Vec<proto::CityLookUpItem>, Self::Error> {
        self.code.detect_error()?;
        Ok(self.location.into_iter().map(Into::into).collect())
    }
}
