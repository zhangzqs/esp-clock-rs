use embedded_svc::{
    http::client::{Client, Connection},
    io::Read,
};

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

impl<C: Connection> WeatherClient<'_, C> {
    pub fn city_lookup<const S: usize>(
        &mut self,
        query: &str,
    ) -> Result<WeatherCityLookupResponse, C::Error> {
        let url = format!("{}/api/weather/city_lookup?query={}", self.base_url, query);
        let req = self.client.get(&url)?;
        let mut resp = req.submit()?;
        let coontent_length = resp.header("Content-Length").unwrap();
        let content_length = coontent_length.parse::<usize>().unwrap();
        if content_length > S {
            panic!("content length: {} > buffer length: {}", content_length, S);
        }
        let mut buf = [0; S];
        let mut buf = &mut buf[..content_length];
        resp.read_exact(&mut buf).unwrap();
        Ok(serde_json::from_slice(&buf).unwrap())
    }

    pub fn now<const S: usize>(&mut self, city_id: &str) -> Result<WeatherNowResponse, C::Error> {
        let url = format!("{}/api/weather/now?city_id={}", self.base_url, city_id);
        let req = self.client.get(&url)?;
        let mut resp = req.submit()?;
        let coontent_length = resp.header("Content-Length").unwrap();
        let content_length = coontent_length.parse::<usize>().unwrap();
        if content_length > S {
            panic!("content length: {} > buffer length: {}", content_length, S);
        }
        let mut buf = [0; S];
        let mut buf = &mut buf[..content_length];
        resp.read_exact(&mut buf).unwrap();
        Ok(serde_json::from_slice(&buf).unwrap())
    }
}
