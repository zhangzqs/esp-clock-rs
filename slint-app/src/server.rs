use std::{
    cell::RefCell,
    fmt::Display,
    marker::PhantomData,
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

    fn new() -> Self;

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
    _phantom: PhantomData<S>,
}

impl<S> HttpServerApp<S>
where
    S: Server<'static>,
{
    pub fn new() -> Self {
        thread::spawn(|| {
            thread::sleep(Duration::from_secs(2));
            let mut server = S::new();
            server
                .fn_handler("/ping", Method::Get, |req| {
                    let mut resp = req.into_ok_response().unwrap();
                    resp.write_all(b"pong").unwrap();
                    Ok(())
                })
                .unwrap();
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        });

        Self {
            _phantom: PhantomData,
        }
    }
}
