use embedded_svc::http::Headers;
use embedded_svc::http::client::{Connection,Status};
use embedded_svc::io::{ Read, Write};
use esp_idf_hal::io::ErrorType;

pub struct SendConnection<T: Connection>(pub T);

unsafe impl<T: Connection> Send for SendConnection<T> {}

impl <T: Connection> Headers for SendConnection<T> {
    fn header(&self, name: &str) -> Option<&'_ str> {
        self.0.header(name)
    }
}

impl<T: Connection> Status for SendConnection<T> {
    fn status(&self) -> u16 {
        self.0.status()
    }

    fn status_message(&self) -> Option<&'_ str> {
        self.0.status_message()
    }
}

impl <T: Connection> ErrorType for SendConnection<T> {
    type Error = T::Error;
}

impl <T: Connection> Read for SendConnection<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.read(buf)
    }
}

impl <T: Connection> Write for SendConnection<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush()
    }
}

impl<T: Connection> Connection for SendConnection<T> {
    type Headers = T::Headers;

    type Read = T::Read;

    type RawConnectionError = T::RawConnectionError;

    type RawConnection = T::RawConnection;

    fn initiate_request<'a>(
        &'a mut self,
        method: embedded_svc::http::Method,
        uri: &'a str,
        headers: &'a [(&'a str, &'a str)],
    ) -> Result<(), Self::Error> {
        self.0.initiate_request(method, uri, headers)
    }

    fn is_request_initiated(&self) -> bool {
        self.0.is_request_initiated()
    }

    fn initiate_response(&mut self) -> Result<(), Self::Error> {
        self.0.initiate_response()
    }

    fn is_response_initiated(&self) -> bool {
        self.0.is_response_initiated()
    }

    fn split(&mut self) -> (&Self::Headers, &mut Self::Read) {
        self.0.split()
    }

    fn raw_connection(&mut self) -> Result<&mut Self::RawConnection, Self::Error> {
        self.0.raw_connection()
    }
}
