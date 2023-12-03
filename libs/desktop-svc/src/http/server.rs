use embedded_io::ErrorType;
use embedded_svc::http::{
    server::{Connection, Handler},
    Headers, Query,
};

pub struct HttpServer {}

#[derive(Debug)]
pub enum HttpServerAdapterConnectionError {
    Other(Box<dyn std::error::Error>),
}

impl embedded_io::Error for HttpServerAdapterConnectionError {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl Connection for HttpServer {
    type Headers = Self;

    type Read = Self;

    type RawConnectionError = HttpServerAdapterConnectionError;

    type RawConnection = Self;

    fn split(&mut self) -> (&Self::Headers, &mut Self::Read) {
        todo!()
    }

    fn initiate_response<'a>(
        &'a mut self,
        status: u16,
        message: Option<&'a str>,
        headers: &'a [(&'a str, &'a str)],
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn is_response_initiated(&self) -> bool {
        todo!()
    }

    fn raw_connection(&mut self) -> Result<&mut Self::RawConnection, Self::Error> {
        todo!()
    }
}

impl embedded_io::ErrorType for HttpServer {
    type Error = HttpServerAdapterConnectionError;
}

impl embedded_io::Read for HttpServer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        todo!()
    }
}

impl embedded_io::Write for HttpServer {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Query for HttpServer {
    fn uri(&self) -> &'_ str {
        todo!()
    }

    fn method(&self) -> embedded_svc::http::Method {
        todo!()
    }
}

impl Headers for HttpServer {
    fn header(&self, name: &str) -> Option<&'_ str> {
        todo!()
    }
}
