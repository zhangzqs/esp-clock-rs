use std::fmt::{Debug, Display};

use embedded_io_adapters::std::ToStd;
use embedded_svc::http::client::{Client, Connection};

use crate::ClientError;

use super::{WeatherCityLookupResponse, WeatherNowResponse};

pub struct WeatherClient<'a, C> {
    base_url: &'a str,
    client: &'a mut Client<C>,
}

impl<'a, C> WeatherClient<'a, C> {
    pub fn new(base_url: &'a str, client: &'a mut Client<C>) -> Self {
        Self { base_url, client }
    }
}

impl<C, E> WeatherClient<'_, C>
where
    C: Connection<Error = E> + Debug,
    E: Debug + Display,
{
    fn get<T>(&mut self, url: &str) -> Result<T, ClientError<C, E>>
    where
        T: serde::de::DeserializeOwned,
    {
        let req = self.client.get(url).map_err(ClientError::Http)?;
        let resp = req.submit().map_err(ClientError::Http)?;
        Ok(serde_json::from_reader(ToStd::new(resp))?)
    }

    pub fn city_lookup(
        &mut self,
        query: &str,
    ) -> Result<WeatherCityLookupResponse, ClientError<C, E>> {
        let url = format!("{}/api/weather/city_lookup?query={}", self.base_url, query);
        self.get(url.as_str())
    }

    pub fn now(&mut self, city_id: &str) -> Result<WeatherNowResponse, ClientError<C, E>> {
        let url = format!("{}/api/weather/now?city_id={}", self.base_url, city_id);
        self.get(url.as_str())
    }
}
