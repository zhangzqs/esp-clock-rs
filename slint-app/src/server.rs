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

use include_dir::{include_dir, Dir};
use log::info;

use embedded_svc::{
    http::{
        server::{Connection, FnHandler, Handler, HandlerResult, Request},
        Method,
    },
    io::Write,
};

static VUE_DIST: Dir = include_dir!("console-dist");

struct VueConsoleHandler;

impl<C> Handler<C> for VueConsoleHandler
where
    C: Connection,
{
    fn handle(&self, c: &mut C) -> HandlerResult {
        let u = c.uri();
        info!("receive http request uri: {}", u);
        let url = url::Url::parse(u)?;

        let file_path = {
            let path = url.path();
            let path = path.strip_prefix("/").unwrap_or(path);
            if path.is_empty() {
                "index.html"
            } else {
                path
            }
        };
        if let Some(f) = VUE_DIST.get_file(file_path) {
            c.initiate_response(200, Some("OK"), &[("Content-Type", "")])?;
            c.write_all(f.contents())?;
        } else {
            c.initiate_response(404, Some("Not Found"), &[("Content-Type", "")])?;
            c.write_all(b"Page Not Found")?;
        }
        Ok(())
    }
}

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
            thread::sleep(Duration::from_secs(10));
            let mut server = S::new();
            server
                .handler("/*", Method::Get, VueConsoleHandler)
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
