use std::time::Duration;

use desktop_svc::http::client::{Configuration, HttpClientConnection, HttpClientConnectionError};

#[derive(Clone, Copy)]
pub struct HttpClientBuilder {
    timeout: Duration,
}

impl slint_app::ClientBuilder for HttpClientBuilder {
    type Conn = HttpClientConnection;
    type HttpClientError = HttpClientConnectionError;

    fn new() -> Self {
        Self {
            timeout: Duration::from_secs(3),
        }
    }

    fn timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    fn build_connection(self) -> Result<Self::Conn, Self::HttpClientError> {
        HttpClientConnection::new(&Configuration {
            timeout: self.timeout,
        })
    }
}
