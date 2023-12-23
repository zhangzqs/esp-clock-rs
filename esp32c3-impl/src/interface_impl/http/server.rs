use std::marker::PhantomData;

use embedded_svc::http::{server::Handler, Method};
use esp_idf_hal::io::EspIOError;
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer};

pub struct EspHttpServerWrapper<'a>(EspHttpServer<'a>);

impl<'a> slint_app::Server<'a> for EspHttpServerWrapper<'a> {
    type Conn<'r> = EspHttpConnection<'r>;
    type HttpServerError = EspIOError;

    fn handler<H>(&mut self, uri: &str, method: Method, handler: H) -> Result<&mut Self, EspIOError>
    where
        H: for<'r> Handler<Self::Conn<'r>> + Send + 'a,
    {
        self.0.handler(uri, method, handler)?;
        Ok(self)
    }
}

#[derive(Clone, Copy)]
pub struct EspHttpServerBuilder<'a> {
    http_port: u16,
    uri_match_wildcard: bool,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> slint_app::ServerBuilder<'a> for EspHttpServerBuilder<'a> {
    type Server = EspHttpServerWrapper<'a>;

    type HttpServerError = EspIOError;

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
        Ok(EspHttpServerWrapper(EspHttpServer::new(&Configuration {
            http_port: self.http_port,
            uri_match_wildcard: self.uri_match_wildcard,
            ..Default::default()
        })?))
    }
}
