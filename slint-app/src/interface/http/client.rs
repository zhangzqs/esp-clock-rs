use std::time::Duration;

use embedded_svc::http::client::Client;

pub trait ClientBuilder: Copy + Clone + Sized {
    type Conn: embedded_svc::http::client::Connection<Error = Self::HttpClientError>;
    type HttpClientError: std::error::Error + std::fmt::Display;

    fn new() -> Self;
    fn timeout(&mut self, timeout: Duration) -> &mut Self;
    fn build_connection(self) -> Result<Self::Conn, Self::HttpClientError>;
    fn build_client(self) -> Result<Client<Self::Conn>, Self::HttpClientError> {
        Ok(Client::wrap(self.build_connection()?))
    }
}
