use embedded_svc::http::{
    server::{Connection, FnHandler, Handler, HandlerResult, Request},
    Method,
};

pub trait ServerBuilder<'a>: Copy + Clone + Sized + 'a {
    type Server: Server<'a, HttpServerError = Self::HttpServerError>;
    type HttpServerError: std::error::Error + std::fmt::Display + Send + Sync;

    fn new() -> Self;
    fn http_port(&mut self, port: u16) -> &mut Self;
    fn uri_match_wildcard(&mut self, enable: bool) -> &mut Self;
    fn build(self) -> Result<Self::Server, Self::HttpServerError>;
}

pub trait Server<'a> {
    type Conn<'r>: Connection;
    type HttpServerError: std::error::Error + Send + Sync;

    fn handler<H>(
        &mut self,
        uri: &str,
        method: Method,
        handler: H,
    ) -> Result<&mut Self, Self::HttpServerError>
    where
        H: for<'r> Handler<Self::Conn<'r>> + Send + 'a;

    fn fn_handler<F>(
        &mut self,
        uri: &str,
        method: Method,
        f: F,
    ) -> Result<&mut Self, Self::HttpServerError>
    where
        F: for<'r> Fn(Request<&mut Self::Conn<'r>>) -> HandlerResult + Send + 'a,
    {
        self.handler(uri, method, FnHandler::new(f))
    }
}
