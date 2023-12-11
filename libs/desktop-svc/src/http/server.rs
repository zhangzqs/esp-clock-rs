use std::{
    collections::HashMap,
    fmt::Display,
    io::{BufRead as _, BufReader, Read, Write},
    net::SocketAddr,
    sync::{atomic::AtomicBool, Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

use embedded_io::Write as _;
use embedded_svc::http::{
    headers::content_type,
    server::{Connection, Handler, HandlerResult},
    Headers, Method, Query,
};
use log::{debug, info, warn};
use rusty_pool::ThreadPool;

struct DefaultHandle404;

impl Handler<HttpServerConnection> for DefaultHandle404 {
    fn handle(&self, req: &mut HttpServerConnection) -> HandlerResult {
        req.initiate_response(404, Some("Not Found"), &[("Content-Type", "text/html")])?;
        req.write_all("<h1>404 Not Found</h1>".as_bytes())?;
        Ok(())
    }
}

pub struct Configuration {
    pub listen_addr: SocketAddr,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:80".parse().unwrap(),
        }
    }
}

pub struct HttpServer<'a> {
    handlers_map: Arc<
        RwLock<
            HashMap<(String, Method), Mutex<Box<dyn Handler<HttpServerConnection> + Send + 'a>>>,
        >,
    >,
    handler_404: Arc<Mutex<Box<dyn Handler<HttpServerConnection> + Send + 'a>>>,
    exit_signal: Arc<AtomicBool>,
    join_handle: Option<thread::JoinHandle<()>>,
    thread_pool: Arc<ThreadPool>,
}

impl<'a: 'static> HttpServer<'a> {
    pub fn new(cfg: Configuration) -> Result<Self, HttpServerError> {
        let listener = std::net::TcpListener::bind(cfg.listen_addr).unwrap();
        listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking");
        let handlers_map: Arc<
            RwLock<HashMap<(String, Method), Mutex<Box<dyn Handler<HttpServerConnection> + Send>>>>,
        > = Arc::new(RwLock::new(HashMap::<_, _>::new()));
        let exit_signal = Arc::new(AtomicBool::new(false));
        let exit_signal_clone = exit_signal.clone();

        let mut res = Self {
            handlers_map: handlers_map.clone(),
            handler_404: Arc::new(Mutex::new(Box::new(DefaultHandle404))),
            exit_signal,
            join_handle: None,
            thread_pool: Arc::new(ThreadPool::default()),
        };

        let handlers_map_clone = handlers_map.clone();
        let handler_404_clone = res.handler_404.clone();
        let thread_pool_clone = res.thread_pool.clone();

        res.join_handle = Some(thread::spawn(move || {
            let handler_404_clone = handler_404_clone.clone();
            let handlers_map_clone = handlers_map_clone.clone();
            let thread_pool_clone = thread_pool_clone.clone();

            for stream in listener.incoming() {
                if exit_signal_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                let handler_404_clone = handler_404_clone.clone();
                let handlers_map_clone = handlers_map_clone.clone();
                let thread_pool_clone = thread_pool_clone.clone();
                match stream {
                    Ok(s) => {
                        // 有新的连接
                        let res = thread_pool_clone.try_execute(move || {
                            let handler_404_clone = handler_404_clone.clone();
                            let handlers_map_clone = handlers_map_clone.clone();

                            let conn = HttpServerConnection::new(s);
                            if let Err(e) = conn {
                                warn!("encountered IO error: {}", e);
                                return;
                            }
                            let mut conn = conn.unwrap();
                            let handlers_map = handlers_map_clone.read().unwrap();
                            let handler = handlers_map.get(&(conn.path.to_string(), conn.method()));

                            let res = if let Some(h) = handler {
                                h.lock().unwrap().handle(&mut conn)
                            } else {
                                handler_404_clone.lock().unwrap().handle(&mut conn)
                            };
                            if let Err(e) = res {
                                conn.handle_error(e);
                            }
                            debug!("request complete");
                        });
                        if let Err(e) = res {
                            warn!("encountered IO error: {}", e);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // 没有新的连接，继续等待
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    Err(e) => panic!("encountered IO error: {e}"),
                };
            }
        }));
        Ok(res)
    }

    pub fn handler<H>(
        &mut self,
        uri: &str,
        method: Method,
        handler: H,
    ) -> Result<&mut Self, HttpServerError>
    where
        H: Handler<HttpServerConnection> + Send + 'a,
    {
        self.handlers_map
            .write()
            .unwrap()
            .insert((uri.to_string(), method), Mutex::new(Box::new(handler)));
        info!("registered handler: {:?} {}", method, uri);
        Ok(self)
    }

    pub fn handle_404<H>(&mut self, handler: H) -> Result<&mut Self, HttpServerError>
    where
        H: Handler<HttpServerConnection> + Send + 'a,
    {
        *self.handler_404.lock().unwrap() = Box::new(handler);
        Ok(self)
    }
}

impl Drop for HttpServer<'_> {
    fn drop(&mut self) {
        self.exit_signal
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.join_handle.take().unwrap().join().unwrap();
    }
}

#[derive(Debug)]
pub enum HttpServerError {
    IO {
        io_error: std::io::Error,
        msg: String,
    },
    Other {
        other_error: Box<dyn std::error::Error>,
        msg: String,
    },
}

impl embedded_io::Error for HttpServerError {
    fn kind(&self) -> embedded_io::ErrorKind {
        if let HttpServerError::IO { io_error, .. } = self {
            return io_error.kind().into();
        }
        embedded_io::ErrorKind::Other
    }
}

impl std::error::Error for HttpServerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HttpServerError::IO { io_error, .. } => Some(io_error),
            HttpServerError::Other { other_error, .. } => Some(other_error.as_ref()),
        }
    }

    fn description(&self) -> &str {
        match self {
            HttpServerError::IO { msg, .. } => msg,
            HttpServerError::Other { msg, .. } => msg,
        }
    }
}

impl Display for HttpServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpServerError::IO { io_error, msg } => {
                write!(f, "IO error: {}, {}", io_error, msg)
            }
            HttpServerError::Other { other_error, msg } => {
                write!(f, "Other error: {}, {}", other_error, msg)
            }
        }
    }
}

pub struct HttpServerConnection {
    stream: std::net::TcpStream,
    is_response_initiated: bool,
    is_complete: bool,
    method: Method,
    uri: String,
    path: String,
    headers: HashMap<String, String>,
}

impl HttpServerConnection {
    pub fn new(stream: std::net::TcpStream) -> Result<Self, HttpServerError> {
        let mut lines = BufReader::new(&stream).lines();
        let mut first_line = if let Some(l) = lines.next() {
            let l = l.unwrap();
            l.split_whitespace()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
        } else {
            return Err(HttpServerError::IO {
                io_error: std::io::ErrorKind::InvalidData.into(),
                msg: "invalid request".to_string(),
            });
        };
        let method = first_line.remove(0);
        let uri = first_line.remove(0);
        let path = uri.split('?').next().unwrap().to_string();
        let version = first_line.remove(0);
        assert_eq!(version, "HTTP/1.1");
        let mut headers = HashMap::new();
        for line in lines {
            let line = line.unwrap();
            if line.is_empty() {
                break;
            }
            let idx = line.find(':');
            if idx.is_none() {
                return Err(HttpServerError::IO {
                    io_error: std::io::ErrorKind::InvalidData.into(),
                    msg: "invalid request".to_string(),
                });
            }
            let kv = line.split_at(idx.unwrap());
            let k = kv.0.to_lowercase();
            let v = kv.1.to_string();
            headers.insert(k, v);
        }

        Ok(Self {
            stream,
            is_response_initiated: false,
            is_complete: false,
            method: {
                match method.as_str() {
                    "GET" => Method::Get,
                    "POST" => Method::Post,
                    "PUT" => Method::Put,
                    "DELETE" => Method::Delete,
                    "HEAD" => Method::Head,
                    "OPTIONS" => Method::Options,
                    "CONNECT" => Method::Connect,
                    "PATCH" => Method::Patch,
                    "TRACE" => Method::Trace,
                    _ => unimplemented!("method: {}", method),
                }
            },
            uri,
            headers,
            path,
        })
    }

    fn handle_error<E>(&mut self, error: E)
    where
        E: Display,
    {
        if self.is_complete {
            warn!(
                "Unhandled internal error [{}], response is already sent",
                error
            );
        } else {
            info!(
                "About to handle internal error [{}], response not sent yet",
                &error
            );

            if let Err(error2) = self.render_error(&error) {
                warn!(
                    "Internal error[{}] while rendering another internal error:\n{}",
                    error2, error
                );
            }
        }
    }

    fn render_error<E>(&mut self, error: E) -> Result<(), HttpServerError>
    where
        E: Display,
    {
        self.initiate_response(500, Some("Internal Error"), &[content_type("text/html")])?;

        self.write_all(
            format!(
                r#"
                    <!DOCTYPE html5>
                    <html>
                        <body style="font-family: Verdana, Sans;">
                            <h1>INTERNAL ERROR</h1>
                            <hr>
                            <pre>{error}</pre>
                        <body>
                    </html>
                "#
            )
            .as_bytes(),
        )?;

        Ok(())
    }
}

impl Connection for HttpServerConnection {
    type Headers = Self;

    type Read = Self;

    type RawConnectionError = HttpServerError;

    type RawConnection = Self;

    fn split(&mut self) -> (&Self::Headers, &mut Self::Read) {
        let headers_ptr: *const Self = self as *const _;
        let headers = unsafe { headers_ptr.as_ref().unwrap() };
        (headers, self)
    }

    fn initiate_response<'a>(
        &'a mut self,
        status: u16,
        message: Option<&'a str>,
        headers: &'a [(&'a str, &'a str)],
    ) -> Result<(), Self::Error> {
        let mut raw = format!("HTTP/1.1 {} {}\r\n", status, message.unwrap_or(""));
        for (k, v) in headers {
            raw.push_str(&format!("{}: {}\r\n", k, v));
        }
        raw.push_str("\r\n");
        self.stream.write_all(raw.as_bytes()).unwrap();
        self.is_response_initiated = true;
        Ok(())
    }

    fn is_response_initiated(&self) -> bool {
        self.is_response_initiated
    }

    fn raw_connection(&mut self) -> Result<&mut Self::RawConnection, Self::Error> {
        Ok(self)
    }
}

impl embedded_io::ErrorType for HttpServerConnection {
    type Error = HttpServerError;
}

impl embedded_io::Read for HttpServerConnection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.stream.read(buf).map_err(|e| HttpServerError::IO {
            io_error: e,
            msg: "read error".to_string(),
        })
    }
}

impl embedded_io::Write for HttpServerConnection {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.stream.write(buf).map_err(|e| HttpServerError::IO {
            io_error: e,
            msg: "write error".to_string(),
        })
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.stream.flush().map_err(|e| HttpServerError::IO {
            io_error: e,
            msg: "flush error".to_string(),
        })
    }
}

impl Query for HttpServerConnection {
    fn uri(&self) -> &'_ str {
        &self.uri
    }

    fn method(&self) -> embedded_svc::http::Method {
        self.method
    }
}

impl Headers for HttpServerConnection {
    fn header(&self, name: &str) -> Option<&'_ str> {
        self.headers
            .get(name.to_lowercase().as_str())
            .map(|s| s.as_str())
    }
}

#[cfg(test)]
mod test_server {
    use std::{net::SocketAddr, thread, time::Duration};

    use embedded_svc::http::{server::FnHandler, Method};

    use super::{Configuration, DefaultHandle404, HttpServer};

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut h = HttpServer::new(Configuration {
            listen_addr: SocketAddr::new(std::net::IpAddr::V4([0, 0, 0, 0].into()), 8080),
        })
        .unwrap();
        h.handler(
            "/test",
            Method::Get,
            FnHandler::new(|req| {
                let body = format!(
                    r#"
                        <h1>HelloWorld</h1> <br>
                        <p>
                            method: {:?} <br>
                            uri: {:?} <br>
                            User-Agent: {:?}
                        </p>
                    "#,
                    req.method(),
                    req.uri(),
                    req.header("User-Agent"),
                )
                .to_string();
                req.into_ok_response()?.write(body.as_bytes())?;
                Ok(())
            }),
        )
        .unwrap()
        .handler(
            "/test1",
            Method::Get,
            FnHandler::new(|_req| Err("error".into())),
        )
        .unwrap()
        .handle_404(DefaultHandle404)
        .unwrap();
        thread::sleep(Duration::from_secs(1000));
        anyhow::Ok(())
    }
}
