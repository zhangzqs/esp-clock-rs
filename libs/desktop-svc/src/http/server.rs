use std::{
    collections::HashMap,
    fmt::Display,
    io::{BufRead as _, BufReader, BufWriter, Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
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
use thiserror::Error;

struct DefaultHandle404;

impl Handler<HttpServerConnection> for DefaultHandle404 {
    fn handle(&self, req: &mut HttpServerConnection) -> HandlerResult {
        req.initiate_response(404, Some("Not Found"), &[("Content-Type", "text/html")])?;
        req.write_all("<h1>404 Not Found</h1>".as_bytes())?;
        Ok(())
    }
}

/// translate from https://github.com/espressif/esp-idf/blob/master/components/esp_http_server/src/httpd_uri.c
fn uri_match_wildcard(template: &str, uri: &str) -> bool {
    let len = uri.len();
    let tpl_len = template.len();
    let mut exact_match_chars = tpl_len;

    /* Check for trailing question mark and asterisk */
    let last = {
        if tpl_len > 0 {
            template.chars().nth(tpl_len - 1).unwrap_or('\0')
        } else {
            '\0'
        }
    };
    let prev_last = {
        if tpl_len > 1 {
            template.chars().nth(tpl_len - 2).unwrap_or('\0')
        } else {
            '\0'
        }
    };
    let asterisk = last == '*' || (prev_last == '*' && last == '?');
    let quest = last == '?' || (prev_last == '?' && last == '*');

    /* Minimum template string length must be:
     *      0 : if neither of '*' and '?' are present
     *      1 : if only '*' is present
     *      2 : if only '?' is present
     *      3 : if both are present
     *
     * The expression (asterisk + quest*2) serves as a
     * case wise generator of these length values
     */

    /* abort in cases such as "?" with no preceding character (invalid template) */
    if exact_match_chars < asterisk as usize + quest as usize * 2 {
        return false;
    }

    /* account for special characters and the optional character if "?" is used */
    exact_match_chars -= asterisk as usize + quest as usize * 2;

    if len < exact_match_chars {
        return false;
    }

    if !quest {
        if !asterisk && len != exact_match_chars {
            /* no special characters and different length - strncmp would return false */
            return false;
        }
        /* asterisk allows arbitrary trailing characters, we ignore these using
         * exact_match_chars as the length limit */
        return &template[..exact_match_chars] == &uri[..exact_match_chars];
    } else {
        /* question mark present */
        if len > exact_match_chars
            && template.chars().nth(exact_match_chars).unwrap()
                != uri.chars().nth(exact_match_chars).unwrap()
        {
            /* the optional character is present, but different */
            return false;
        }
        if &template[..exact_match_chars] != &uri[..exact_match_chars] {
            /* the mandatory part differs */
            return false;
        }
        /* Now we know the URI is longer than the required part of template,
         * the mandatory part matches, and if the optional character is present, it is correct.
         * Match is OK if we have asterisk, i.e. any trailing characters are OK, or if
         * there are no characters beyond the optional character. */
        return asterisk || len <= exact_match_chars + 1;
    }
}

#[test]
fn test_uri_match_wildcard() {
    let test_cases = [
        ["/", "/", "true"],
        ["", "", "true"],
        ["/", "", "false"],
        ["/wrong", "/", "false"],
        ["/", "/wrong", "false"],
        [
            "/asdfghjkl/qwertrtyyuiuioo",
            "/asdfghjkl/qwertrtyyuiuioo",
            "true",
        ],
        ["/path", "/path", "true"],
        ["/path", "/path/", "false"],
        ["/path/", "/path", "false"],
        ["?", "", "false"], // this is not valid, but should not crash
        ["?", "sfsdf", "false"],
        ["/path/?", "/pa", "false"],
        ["/path/?", "/path", "true"],
        ["/path/?", "/path/", "true"],
        ["/path/?", "/path/alalal", "false"],
        ["/path/*", "/path", "false"],
        ["/path/*", "/", "false"],
        ["/path/*", "/path/", "true"],
        ["/path/*", "/path/blabla", "true"],
        ["*", "", "true"],
        ["*", "/", "true"],
        ["*", "/aaa", "true"],
        ["/path/?*", "/pat", "false"],
        ["/path/?*", "/pathb", "false"],
        ["/path/?*", "/pathxx", "false"],
        ["/path/?*", "/pathblabla", "false"],
        ["/path/?*", "/path", "true"],
        ["/path/?*", "/path/", "true"],
        ["/path/?*", "/path/blabla", "true"],
        ["/path/*?", "/pat", "false"],
        ["/path/*?", "/pathb", "false"],
        ["/path/*?", "/pathxx", "false"],
        ["/path/*?", "/path", "true"],
        ["/path/*?", "/path/", "true"],
        ["/path/*?", "/path/blabla", "true"],
        ["/path/*/xxx", "/path/", "false"],
        ["/path/*/xxx", "/path/*/xxx", "true"],
    ];

    for test_case in test_cases.iter() {
        let res = uri_match_wildcard(test_case[0], test_case[1]);
        let res = if res { "true" } else { "false" };
        assert_eq!(res, test_case[2], "test case: {:?}", test_case);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Configuration {
    pub http_port: u16,
    pub uri_match_wildcard: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            http_port: 80,
            uri_match_wildcard: false,
        }
    }
}

pub struct HttpServer<'a> {
    handlers_map: Arc<
        RwLock<
            HashMap<
                Method,
                HashMap<String, Mutex<Box<dyn Handler<HttpServerConnection> + Send + 'a>>>,
            >,
        >,
    >,
    handler_404: Arc<Mutex<Box<dyn Handler<HttpServerConnection> + Send + 'a>>>,
    exit_signal: Arc<AtomicBool>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl HttpServer<'static> {
    pub fn new(conf: &Configuration) -> Result<Self, HttpServerError> {
        let listener = std::net::TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(0, 0, 0, 0),
            conf.http_port,
        )))
        .unwrap();
        listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking");
        let handlers_map = Arc::new(RwLock::new(HashMap::new()));
        let exit_signal = Arc::new(AtomicBool::new(false));
        let exit_signal_clone = exit_signal.clone();

        let mut res = Self {
            handlers_map: handlers_map.clone(),
            handler_404: Arc::new(Mutex::new(Box::new(DefaultHandle404))),
            exit_signal,
            join_handle: None,
        };

        let handlers_map_clone = handlers_map.clone();
        let handler_404_clone = res.handler_404.clone();

        let uri_match_wildcard_enable = conf.uri_match_wildcard;
        res.join_handle = Some(thread::spawn(move || {
            let handler_404_clone = handler_404_clone.clone();
            let handlers_map_clone = handlers_map_clone.clone();

            for stream in listener.incoming() {
                if exit_signal_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                let handler_404_clone = handler_404_clone.clone();
                let handlers_map_clone = handlers_map_clone.clone();
                match stream {
                    Ok(s) => {
                        // 有新的连接
                        let handler_404_clone = handler_404_clone.clone();
                        let handlers_map_clone = handlers_map_clone.clone();

                        let conn = HttpServerConnection::new(s);
                        if let Err(e) = conn {
                            warn!("encountered IO error: {}", e);
                            continue;
                        }
                        let mut conn = conn.unwrap();
                        let handlers_map = handlers_map_clone.read().unwrap();
                        let path_headers = handlers_map.get(&conn.method());
                        if path_headers.is_none() {
                            conn.handle_error("method not allowed");
                            continue;
                        }
                        let path_headers = path_headers.unwrap();

                        let mut handler = None;
                        for (path, h) in path_headers {
                            if uri_match_wildcard_enable {
                                if !uri_match_wildcard(path, &conn.path) {
                                    continue;
                                }
                            } else {
                                if path != &conn.path {
                                    continue;
                                }
                            }
                            handler = Some(h);
                            break;
                        }

                        let res = if let Some(h) = handler {
                            h.lock().unwrap().handle(&mut conn)
                        } else {
                            handler_404_clone.lock().unwrap().handle(&mut conn)
                        };
                        if let Err(e) = res {
                            conn.handle_error(e);
                        }
                        debug!("request complete");
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
}

impl<'a> HttpServer<'a> {
    pub fn handler<H>(
        &mut self,
        uri: &str,
        method: Method,
        handler: H,
    ) -> Result<&mut Self, HttpServerError>
    where
        H: Handler<HttpServerConnection> + Send + 'a,
    {
        let mut handlers_map = self.handlers_map.write().unwrap();
        let path_headers = handlers_map.entry(method).or_insert_with(HashMap::new);
        path_headers.insert(uri.to_string(), Mutex::new(Box::new(handler)));
        drop(handlers_map);
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

#[derive(Debug, Error)]
pub enum HttpServerError {
    #[error("IO error: {io_error}, {msg}")]
    IO {
        io_error: std::io::Error,
        msg: String,
    },
    #[error("other error")]
    Other {
        msg: String,
    }
}

impl From<std::io::Error> for HttpServerError {
    fn from(io_error: std::io::Error) -> Self {
        Self::IO {
            msg: io_error.to_string(),
            io_error,
        }
    }
}

impl embedded_io::Error for HttpServerError {
    fn kind(&self) -> embedded_io::ErrorKind {
        if let HttpServerError::IO { io_error, .. } = self {
            return io_error.kind().into();
        }
        embedded_io::ErrorKind::Other
    }
}

pub struct HttpServerConnection {
    reader: BufReader<std::net::TcpStream>,
    writer: BufWriter<std::net::TcpStream>,
    is_response_initiated: bool,
    is_complete: bool,
    method: Method,
    uri: String,
    path: String,
    headers: HashMap<String, String>,
}

impl HttpServerConnection {
    pub fn new(stream: std::net::TcpStream) -> Result<Self, HttpServerError> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream.try_clone()?);
        let (method, uri, path) = {
            let mut buf = String::new();
            if let Err(e) = reader.read_line(&mut buf) {
                return Err(HttpServerError::IO {
                    io_error: std::io::ErrorKind::InvalidData.into(),
                    msg: format!("invalid request: {}", e),
                });
            }
            let mut first_line = buf
                .split_whitespace()
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            let method = first_line.remove(0);
            let uri = first_line.remove(0);
            let path = uri.split('?').next().unwrap().to_string();
            let version = first_line.remove(0);
            assert_eq!(version, "HTTP/1.1");
            (method, uri, path)
        };
        let headers = {
            let mut headers = HashMap::new();
            loop {
                let mut line = String::new();
                if let Err(e) = reader.read_line(&mut line) {
                    return Err(HttpServerError::IO {
                        io_error: std::io::ErrorKind::InvalidData.into(),
                        msg: format!("invalid request: {}", e),
                    });
                }
                let line = line.trim();
                if line.is_empty() {
                    break;
                }
                let idx = line.find(':');
                if idx.is_none() {
                    return Err(HttpServerError::IO {
                        io_error: std::io::ErrorKind::InvalidData.into(),
                        msg: format!("invalid request: {}", line),
                    });
                }
                let kv = line.split_at(idx.unwrap());
                let k = kv.0.to_lowercase();
                let v = kv.1.to_string();
                headers.insert(k, v);
            }
            headers
        };

        Ok(Self {
            reader,
            writer,
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
        if let Err(e) = self.writer.write_all(raw.as_bytes()) {
            return Err(HttpServerError::IO {
                io_error: e,
                msg: "write response header error".to_string(),
            });
        }
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
        self.reader.read(buf).map_err(|e| HttpServerError::IO {
            io_error: e,
            msg: "read request body error".to_string(),
        })
    }
}

impl embedded_io::Write for HttpServerConnection {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.writer.write(buf).map_err(|e| HttpServerError::IO {
            io_error: e,
            msg: "write response body error".to_string(),
        })
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.writer.flush().map_err(|e| HttpServerError::IO {
            io_error: e,
            msg: "flush response body error".to_string(),
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
    use std::{thread, time::Duration};

    use embedded_io::Write;
    use embedded_svc::http::{server::FnHandler, Method};
    use log::debug;

    use super::{Configuration, DefaultHandle404, HttpServer};

    #[test]
    fn test_post() -> anyhow::Result<()> {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::init();
        let mut h = HttpServer::new(&Configuration {
            http_port: 8081,
            uri_match_wildcard: true,
        })?;
        h.handler(
            "/ping",
            Method::Post,
            FnHandler::new(|mut req| {
                let mut buf = [0u8; 4];
                req.read(&mut buf)?;
                debug!("read: {:?}", buf);
                let mut resp = req.into_ok_response()?;
                resp.write_all(b"pong")?;
                Ok(())
            }),
        )?
        .handle_404(DefaultHandle404)?;
        thread::sleep(Duration::from_secs(1000));
        anyhow::Ok(())
    }

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut h = HttpServer::new(&Configuration {
            http_port: 8080,
            uri_match_wildcard: true,
        })?;
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
        )?
        .handler(
            "/test1",
            Method::Get,
            FnHandler::new(|_req| Err("error".into())),
        )?
        .handler(
            "/t1/*",
            Method::Get,
            FnHandler::new(|req| {
                let uri = req.uri().to_string();
                req.into_ok_response()?.write_all(uri.as_bytes())?;
                Ok(())
            }),
        )?
        .handle_404(DefaultHandle404)?;
        thread::sleep(Duration::from_secs(1000));
        anyhow::Ok(())
    }
}
