use embedded_io::ErrorType;
use embedded_io::{Read, Write};
use embedded_svc::http::client::Connection;
use embedded_svc::http::Method;
use embedded_svc::http::Status;
use reqwest;

struct NewType<T>(pub T);

impl From<embedded_svc::http::Method> for NewType<reqwest::Method> {
    fn from(method: Method) -> Self {
        let m = match method {
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
        };
        Self(m)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum State {
    New,
    Request,
    Response,
}

struct HttpClientAdapterConnection {
    client: reqwest::Client,
    state: State,
}

impl HttpClientAdapterConnection {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            state: State::New,
        }
    }

    fn assert_initial(&self) {
        if self.state != State::New && self.state != State::Response {
            panic!("connection is not in initial phase");
        }
    }

    fn assert_request(&self) {
        if self.state != State::Request {
            panic!("connection is not in request phase");
        }
    }

    fn assert_response(&self) {
        if self.state != State::Response {
            panic!("connection is not in response phase");
        }
    }
}

impl embedded_svc::http::Headers for HttpClientAdapterConnection {
    fn header(&self, name: &str) -> Option<&'_ str> {
        todo!()
    }
}

impl Status for HttpClientAdapterConnection {
    fn status(&self) -> u16 {
        todo!()
    }

    fn status_message(&self) -> Option<&'_ str> {
        todo!()
    }
}

#[derive(Debug)]
struct HttpClientAdapterConnectionError {
    error: reqwest::Error,
}

impl embedded_io::Error for HttpClientAdapterConnectionError {
    fn kind(&self) -> embedded_io::ErrorKind {
        todo!()
    }
}

impl ErrorType for HttpClientAdapterConnection {
    type Error = HttpClientAdapterConnectionError;
}

impl Read for HttpClientAdapterConnection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        todo!()
    }
}

impl Write for HttpClientAdapterConnection {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Connection for HttpClientAdapterConnection {
    type Headers = Self;

    type Read = Self;

    type RawConnectionError = HttpClientAdapterConnectionError;

    type RawConnection = Self;

    fn initiate_request<'a>(
        &'a mut self,
        method: Method,
        uri: &'a str,
        headers: &'a [(&'a str, &'a str)],
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn is_request_initiated(&self) -> bool {
        todo!()
    }

    fn initiate_response(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    fn is_response_initiated(&self) -> bool {
        todo!()
    }

    fn split(&mut self) -> (&Self::Headers, &mut Self::Read) {
        todo!()
    }

    fn raw_connection(&mut self) -> Result<&mut Self::RawConnection, Self::Error> {
        todo!()
    }
}
