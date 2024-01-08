use std::fmt::{Debug, Display};

use embedded_svc::{
    http::client::{Client, Connection},
    io::Read,
};

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
    fn get<T, const S: usize>(&mut self, url: &str) -> Result<T, ClientError<C, E>>
    where
        T: serde::de::DeserializeOwned,
    {
        let req = self.client.get(url).map_err(ClientError::Http)?;
        let mut resp = req.submit().map_err(ClientError::Http)?;
        let coontent_length = resp
            .header("Content-Length")
            .ok_or(ClientError::UnknownContentLength(None))?;
        let content_length = coontent_length
            .parse::<usize>()
            .map_err(|_| ClientError::UnknownContentLength(Some(coontent_length.to_string())))?;
        if content_length > S {
            return Err(ClientError::BufferTooSmall {
                content_length,
                buffer_length: S,
            });
        }
        let mut buf = [0u8; S];
        let mut buf = &mut buf[..content_length];
        resp.read_exact(&mut buf)?;
        Ok(serde_json::from_slice(&buf)?)
    }

    pub fn city_lookup<const S: usize>(
        &mut self,
        query: &str,
    ) -> Result<WeatherCityLookupResponse, ClientError<C, E>> {
        let url = format!("{}/api/weather/city_lookup?query={}", self.base_url, query);
        self.get::<_, S>(url.as_str())
    }

    pub fn now<const S: usize>(
        &mut self,
        city_id: &str,
    ) -> Result<WeatherNowResponse, ClientError<C, E>> {
        let url = format!("{}/api/weather/now?city_id={}", self.base_url, city_id);
        self.get::<_, S>(url.as_str())
    }
}
