pub struct HttpClientConnection {}

impl HttpClientConnection {
    pub fn new() -> Self {
        Self {}
    }
}

impl embedded_svc::http::Headers for HttpClientConnection {
    fn header(&self, name: &str) -> Option<&'_ str> {
        todo!()
    }
}

impl embedded_svc::http::Status for HttpClientConnection {
    fn status(&self) -> u16 {
        todo!()
    }

    fn status_message(&self) -> Option<&'_ str> {
        todo!()
    }
}

#[derive(Debug)]
pub enum HttpClientConnectionError {}

impl embedded_io::Error for HttpClientConnectionError {
    fn kind(&self) -> embedded_io::ErrorKind {
        todo!()
    }
}

impl std::fmt::Display for HttpClientConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for HttpClientConnectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        todo!()
    }
}

impl embedded_io::ErrorType for HttpClientConnection {
    type Error = HttpClientConnectionError;
}

impl embedded_io::Read for HttpClientConnection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        todo!()
    }
}

impl embedded_io::Write for HttpClientConnection {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl embedded_svc::http::client::Connection for HttpClientConnection {
    type Headers = Self;

    type Read = Self;

    type RawConnectionError = HttpClientConnectionError;

    type RawConnection = Self;

    fn initiate_request<'a>(
        &'a mut self,
        method: embedded_svc::http::Method,
        uri: &'a str,
        headers: &'a [(&'a str, &'a str)],
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn is_request_initiated(&self) -> bool {
        false
    }

    fn initiate_response(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    fn is_response_initiated(&self) -> bool {
        false
    }

    fn split(&mut self) -> (&Self::Headers, &mut Self::Read) {
        todo!()
    }

    fn raw_connection(&mut self) -> Result<&mut Self::RawConnection, Self::Error> {
        todo!()
    }
}
