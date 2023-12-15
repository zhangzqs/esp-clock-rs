use std::{
    cell::RefCell,
    fmt::Display,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use log::info;

use embedded_svc::{
    http::{
        server::{Connection, FnHandler, Handler, HandlerError, HandlerResult, Request},
        Method,
    },
    io::Write,
};

pub trait Server<'a> {
    type Conn<'r>: Connection;
    type HttpServerError: std::error::Error;

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

pub struct HttpServerApp<S>
where
    S: Server<'static>,
{
    server: S,
}

impl<S> HttpServerApp<S>
where
    S: Server<'static>,
{
    pub fn new(server: S) -> Self {
        let mut s = Self { server: server };
        s.bind();
        s
    }

    fn bind(&mut self) {
        self.server
            .fn_handler("/ping", Method::Get, |req| {
                let mut resp = req.into_ok_response().unwrap();
                resp.write_all(b"pong").unwrap();
                Ok(())
            })
            .unwrap();
    }
}
