use std::net::SocketAddr;

use embedded_io::ErrorType;
use embedded_svc::http::{
    server::{Connection, FnHandler, Handler, HandlerResult, Request},
    Headers, Method, Query,
};
use hyper::client::conn::http1;
use tokio::net::TcpListener;
use hyper::service::service_fn;

pub struct HttpServer {}

impl HttpServer {
    pub fn new() -> Result<Self, HttpServerAdapterError> {
        let addr: SocketAddr = ([127, 0, 0, 1], 1338).into();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let listener = TcpListener::bind(addr).await.unwrap();
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                tokio::spawn(async {
                    let h1 = http1::Builder::new();
                });
            }
        });

        todo!()
    }

    pub fn handler<H>(
        &mut self,
        uri: &str,
        method: Method,
        handler: H,
    ) -> Result<&mut Self, HttpServerAdapterError>
    where
        H: Handler<HttpServerConnection> + Send,
    {
        todo!()
    }

    pub fn fn_handler<F>(
        &mut self,
        uri: &str,
        method: Method,
        f: F,
    ) -> Result<&mut Self, HttpServerAdapterError>
    where
        F: Fn(Request<&mut HttpServerConnection>) -> HandlerResult + Send,
    {
        self.handler(uri, method, FnHandler::new(f))
    }

    pub fn stop(&mut self) -> Result<(), HttpServerAdapterError> {
        todo!()
    }
}

pub struct HttpServerConnection {
    request: HttpServerRawConnection,
}

#[derive(Debug)]
pub enum HttpServerAdapterError {
    Other(Box<dyn std::error::Error>),
}

impl embedded_io::Error for HttpServerAdapterError {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl Connection for HttpServerConnection {
    type Headers = Self;

    type Read = Self;

    type RawConnectionError = HttpServerAdapterError;

    type RawConnection = HttpServerRawConnection;

    fn split(&mut self) -> (&Self::Headers, &mut Self::Read) {
        todo!()
    }

    fn initiate_response<'a>(
        &'a mut self,
        _status: u16,
        _message: Option<&'a str>,
        _headers: &'a [(&'a str, &'a str)],
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn is_response_initiated(&self) -> bool {
        todo!()
    }

    fn raw_connection(&mut self) -> Result<&mut Self::RawConnection, Self::Error> {
        Ok(&mut self.request)
    }
}

impl embedded_io::ErrorType for HttpServerConnection {
    type Error = HttpServerAdapterError;
}

impl embedded_io::Read for HttpServerConnection {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        todo!()
    }
}

impl embedded_io::Write for HttpServerConnection {
    fn write(&mut self, _buf: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Query for HttpServerConnection {
    fn uri(&self) -> &'_ str {
        todo!()
    }

    fn method(&self) -> embedded_svc::http::Method {
        todo!()
    }
}

impl Headers for HttpServerConnection {
    fn header(&self, _name: &str) -> Option<&'_ str> {
        todo!()
    }
}

pub struct HttpServerRawConnection {
    // req
}

impl embedded_io::ErrorType for HttpServerRawConnection {
    type Error = HttpServerAdapterError;
}

impl embedded_io::Read for HttpServerRawConnection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        todo!()
    }
}

impl embedded_io::Write for HttpServerRawConnection {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod test_server {
    use std::{thread, time::Duration};

    use embedded_svc::http::Method;

    use super::HttpServer;

    #[test]
    fn test() {
        let mut h = HttpServer::new().unwrap();
        h.fn_handler("/test", Method::Get, |r| {
            Ok(())
        });
        thread::sleep(Duration::from_secs(10));
        h.stop().unwrap();
    }
}