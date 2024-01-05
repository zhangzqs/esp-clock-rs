use std::fmt::{Debug, Display};

use embedded_svc::{
    http::client::{Client, Connection},
    io::Read,
};
use serde::Deserialize;

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

#[derive(thiserror::Error, Debug)]
pub enum WeatherError<C, E>
where
    C: Connection<Error = E>,
    E: Display,
{
    #[error("http error: {0}")]
    Http(C::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl<C, E> WeatherClient<'_, C>
where
    C: Connection<Error = E>,
    E: Debug + Display,
{
    fn get<T, const S: usize>(&mut self, url: &str) -> Result<T, WeatherError<C, E>>
    where
        T: serde::de::DeserializeOwned,
    {
        let req = self.client.get(url).map_err(WeatherError::Http)?;
        let mut resp = req.submit().map_err(WeatherError::Http)?;
        let coontent_length = resp.header("Content-Length").unwrap();
        let content_length = coontent_length.parse::<usize>().unwrap();
        if content_length > S {
            panic!("content length: {} > buffer length: {}", content_length, S);
        }
        let mut buf = [0u8; S];
        let mut buf = &mut buf[..content_length];
        resp.read_exact(&mut buf).unwrap();
        Ok(serde_json::from_slice(&buf)?)
    }

    pub fn city_lookup<const S: usize>(
        &mut self,
        query: &str,
    ) -> Result<WeatherCityLookupResponse, WeatherError<C, E>> {
        let url = format!("{}/api/weather/city_lookup?query={}", self.base_url, query);
        self.get::<_, S>(url.as_str())
    }

    pub fn now<const S: usize>(
        &mut self,
        city_id: &str,
    ) -> Result<WeatherNowResponse, WeatherError<C, E>> {
        let url = format!("{}/api/weather/now?city_id={}", self.base_url, city_id);
        self.get::<_, S>(url.as_str())
    }
}
