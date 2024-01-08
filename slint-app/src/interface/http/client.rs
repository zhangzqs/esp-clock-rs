use std::{
    error::Error,
    fmt::{Debug, Display},
    time::Duration,
};

use embedded_svc::http::client::{Client, Connection};

pub trait ClientBuilder: Copy + Clone + Sized {
    type Conn: Connection<Error = Self::HttpClientError> + Debug;
    type HttpClientError: Error + Display;

    fn new() -> Self;
    fn timeout(&mut self, timeout: Duration) -> &mut Self;
    fn build_connection(self) -> Result<Self::Conn, Self::HttpClientError>;
    fn build_client(self) -> Result<Client<Self::Conn>, Self::HttpClientError> {
        Ok(Client::wrap(self.build_connection()?))
    }
}
