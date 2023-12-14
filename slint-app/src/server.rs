use std::{
    cell::RefCell,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use embedded_svc::{
    http::{
        server::{Connection, FnHandler, Handler, HandlerResult, Request},
        Method,
    },
    io::Write,
};

use desktop_svc::http::server::*;

pub struct HttpServerApp {
    server: Arc<Mutex<Option<HttpServer<'static>>>>,
}

impl HttpServerApp {
    pub fn new() -> Self {
        let server = Arc::new(Mutex::new(None));
        let server_ref = server.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(10));
            let mut server = HttpServer::new(Configuration {
                listen_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8080)),
            })
            .unwrap();
            server
                .handler(
                    "/ping",
                    Method::Get,
                    FnHandler::new(|req| {
                        let mut resp = req.into_ok_response().unwrap();
                        resp.write_all(b"pong").unwrap();
                        Ok(())
                    }),
                )
                .unwrap();
            *server_ref.lock().unwrap() = Some(server);
        });
        Self {
            server: server.clone(),
        }
    }
}
