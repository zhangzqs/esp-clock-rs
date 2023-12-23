use std::marker::PhantomData;

use desktop_svc::http::server::{Configuration, HttpServer, HttpServerError};
use embedded_svc::http::{server::Handler, Method};
pub struct HttpServerWrapper<'a>(HttpServer<'a>);

impl<'a: 'static> slint_app::Server<'a> for HttpServerWrapper<'a> {
    type Conn<'r> = desktop_svc::http::server::HttpServerConnection;
    type HttpServerError = desktop_svc::http::server::HttpServerError;

    fn handler<H>(
        &mut self,
        uri: &str,
        method: Method,
        handler: H,
    ) -> Result<&mut Self, Self::HttpServerError>
    where
        H: for<'r> Handler<Self::Conn<'r>> + Send + 'a,
    {
        self.0.handler(uri, method, handler)?;
        Ok(self)
    }
}

#[derive(Clone, Copy)]
pub struct HttpServerBuilder<'a> {
    http_port: u16,
    uri_match_wildcard: bool,
    _phantom: PhantomData<&'a ()>,
}

impl<'a: 'static> slint_app::ServerBuilder<'a> for HttpServerBuilder<'a> {
    type Server = HttpServerWrapper<'a>;

    type HttpServerError = HttpServerError;

    fn new() -> Self {
        Self {
            http_port: 8080,
            uri_match_wildcard: false,
            _phantom: PhantomData,
        }
    }

    fn http_port(&mut self, port: u16) -> &mut Self {
        self.http_port = port;
        self
    }

    fn uri_match_wildcard(&mut self, enable: bool) -> &mut Self {
        self.uri_match_wildcard = enable;
        self
    }

    fn build(self) -> Result<Self::Server, Self::HttpServerError> {
        Ok(HttpServerWrapper(HttpServer::new(&Configuration {
            http_port: self.http_port,
            uri_match_wildcard: self.uri_match_wildcard,
        })?))
    }
}
