use std::error::Error;
use std::io::Read;
use std::str::FromStr;

use embedded_io::{ErrorType, Write as _};
use embedded_svc::http::client::Connection;
use embedded_svc::http::Method;
use embedded_svc::http::Status;
use reqwest;
use reqwest::header::{HeaderMap, HeaderName};

fn method_type_convert(method: Method) -> reqwest::Method {
    match method {
        Method::Get => reqwest::Method::GET,
        Method::Post => reqwest::Method::POST,
        Method::Put => reqwest::Method::PUT,
        Method::Delete => reqwest::Method::DELETE,
        Method::Head => reqwest::Method::HEAD,
        Method::Options => reqwest::Method::OPTIONS,
        Method::Connect => reqwest::Method::CONNECT,
        Method::Patch => reqwest::Method::PATCH,
        Method::Trace => reqwest::Method::TRACE,
        _ => panic!("Unsupported method: {:?}", method),
    }
}

pub struct HttpClientConnection {
    client: reqwest::blocking::Client,
    request: Option<reqwest::blocking::RequestBuilder>,
    response: Option<reqwest::blocking::Response>,
    req_buffer: Vec<u8>,
}

pub struct Configuration {
    pub timeout: std::time::Duration,
}

impl HttpClientConnection {
    pub fn new(configuration: &Configuration) -> Result<Self, HttpClientConnectionError> {
        Ok(Self {
            client: reqwest::blocking::Client::builder()
                .timeout(configuration.timeout)
                .build()?,
            request: None,
            response: None,
            req_buffer: Vec::new(),
        })
    }

    fn assert_request(&self) {
        if self.request.is_none() {
            panic!("connection is not in request phase");
        }
    }

    fn assert_response(&self) {
        if self.response.is_none() {
            panic!("connection is not in response phase");
        }
    }
}

impl embedded_svc::http::Headers for HttpClientConnection {
    fn header(&self, name: &str) -> Option<&'_ str> {
        self.assert_response();
        self.response
            .as_ref()
            .unwrap()
            .headers()
            .get(name)
            .map(|v| v.to_str().unwrap())
    }
}

impl Status for HttpClientConnection {
    fn status(&self) -> u16 {
        self.assert_response();
        self.response.as_ref().unwrap().status().as_u16()
    }

    fn status_message(&self) -> Option<&'_ str> {
        self.assert_response();
        self.response.as_ref().unwrap().status().canonical_reason()
    }
}

#[derive(Debug)]
pub enum HttpClientConnectionError {
    ReqwestError(reqwest::Error),
    Other(Box<dyn std::error::Error>),
}

impl From<Box<dyn std::error::Error>> for HttpClientConnectionError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::Other(error)
    }
}

impl From<reqwest::Error> for HttpClientConnectionError {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(error)
    }
}

impl embedded_io::Error for HttpClientConnectionError {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl std::fmt::Display for HttpClientConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpClientConnectionError::ReqwestError(e) => e.fmt(f),
            HttpClientConnectionError::Other(e) => e.fmt(f),
        }
    }
}

impl Error for HttpClientConnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReqwestError(e) => Some(e),
            Self::Other(e) => Some(e.as_ref()),
        }
    }
}

impl ErrorType for HttpClientConnection {
    type Error = HttpClientConnectionError;
}

impl embedded_io::Read for HttpClientConnection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.assert_response();
        let u = self.response.as_mut().unwrap().read(buf).unwrap();
        Ok(u)
    }
}

impl embedded_io::Write for HttpClientConnection {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.assert_request();
        self.req_buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.assert_request();
        self.request = Some(self.request.take().unwrap().body(self.req_buffer.clone()));
        Ok(())
    }
}

impl Connection for HttpClientConnection {
    type Headers = Self;

    type Read = Self;

    type RawConnectionError = HttpClientConnectionError;

    type RawConnection = Self;

    fn initiate_request<'a>(
        &'a mut self,
        method: Method,
        uri: &'a str,
        headers: &'a [(&'a str, &'a str)],
    ) -> Result<(), Self::Error> {
        let mut hs = HeaderMap::new();
        for (k, v) in headers {
            let k = HeaderName::from_str(k).unwrap();
            let v = v.parse().unwrap();
            hs.insert(k, v);
        }
        self.request = Some(
            self.client
                .request(method_type_convert(method), uri)
                .headers(hs),
        );
        Ok(())
    }

    fn is_request_initiated(&self) -> bool {
        self.request.is_some()
    }

    fn initiate_response(&mut self) -> Result<(), Self::Error> {
        self.assert_request();
        self.flush()?;

        let req_builder = self.request.take().unwrap();
        let req = req_builder.build()?;
        let resp = self.client.execute(req)?;
        self.response = Some(resp);
        Ok(())
    }

    fn is_response_initiated(&self) -> bool {
        self.response.is_some()
    }

    fn split(&mut self) -> (&Self::Headers, &mut Self::Read) {
        todo!("unimplemented")
    }

    fn raw_connection(&mut self) -> Result<&mut Self::RawConnection, Self::Error> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use embedded_svc::{http::client::Client, utils::io};

    use super::*;

    #[test]
    fn test_http_client_adapter() {
        let conn = HttpClientConnection::new().unwrap();
        // Prepare headers and URL
        let headers = [("accept", "text/plain")];
        let url: &str = "http://ifconfig.net/";
        let mut client = Client::<HttpClientConnection>::wrap(conn);
        let req = client.request(Method::Get, url, &headers).unwrap();
        println!("-> GET {}", url);
        let mut resp = req.submit().unwrap();
        println!("<- {} {}", resp.status(), resp.status_message().unwrap());
        assert_eq!(resp.status(), 200);
        let mut buf = [0u8; 1024];
        let br = io::try_read_full(&mut resp, &mut buf).unwrap();
        println!("Read {} bytes", br);
        let s = std::str::from_utf8(&buf[..br]).unwrap();
        println!("Response body (truncated to {} bytes): {:?}", buf.len(), s);
        // Drain the remaining response bytes
        while resp.read(&mut buf).unwrap() > 0 {}
    }

    #[test]
    fn test_http_client_adapter_repeat() {
        let conn = HttpClientConnection::new().unwrap();
        // Prepare headers and URL
        let headers = [("accept", "text/plain")];
        let url: &str = "http://ifconfig.net/";
        let mut client = Client::<HttpClientConnection>::wrap(conn);
        let req = client.request(Method::Get, url, &headers).unwrap();
        let resp = req.submit().unwrap();
        println!("<- {} {}", resp.status(), resp.status_message().unwrap());
        let req = client.request(Method::Get, url, &headers).unwrap();
        let resp = req.submit().unwrap();
        println!("<- {} {}", resp.status(), resp.status_message().unwrap());
    }
}
