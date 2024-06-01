use embedded_svc::http::client::{Connection, Status};
use embedded_svc::http::Headers;
use embedded_svc::io::{Read, Write};
use esp_idf_hal::io::{ErrorType, EspIOError};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use esp_idf_sys::{EspError, ESP_ERR_HTTP_EAGAIN, ESP_FAIL};
use log::error;
use std::fmt::Debug;
use std::result::Result;
use std::time::Duration;

pub struct MyConnection {
    config: Configuration,
    conn: EspHttpConnection,
    need_build_new_connection: bool,
}

impl Debug for MyConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyConnection")
            .field("config", &self.config)
            .field("conn", &"EspHttpConnection")
            .field("need_build_new_connection", &self.need_build_new_connection)
            .finish()
    }
}

unsafe impl Send for MyConnection {}

impl MyConnection {
    pub fn new(config: Configuration) -> Result<Self, EspIOError> {
        Ok(Self {
            config,
            conn: EspHttpConnection::new(&config)?,
            need_build_new_connection: false,
        })
    }
}

impl Headers for MyConnection {
    fn header(&self, name: &str) -> Option<&'_ str> {
        self.conn.header(name)
    }
}

impl Status for MyConnection {
    fn status(&self) -> u16 {
        self.conn.status()
    }

    fn status_message(&self) -> Option<&'_ str> {
        self.conn.status_message()
    }
}

impl ErrorType for MyConnection {
    type Error = EspIOError;
}

impl Read for MyConnection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.conn.read(buf).map_err(EspIOError)
    }
}

impl Write for MyConnection {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.conn.write(buf).map_err(EspIOError)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.conn.flush()
    }
}

impl Connection for MyConnection {
    type Headers = Self;

    type Read = Self;

    type RawConnectionError = EspIOError;

    type RawConnection = Self;

    fn initiate_request<'a>(
        &'a mut self,
        method: embedded_svc::http::Method,
        uri: &'a str,
        headers: &'a [(&'a str, &'a str)],
    ) -> Result<(), Self::Error> {
        if self.need_build_new_connection {
            self.conn = EspHttpConnection::new(&self.config)?;
            self.need_build_new_connection = false;
        }
        self.conn
            .initiate_request(method, uri, headers)
            .map_err(EspIOError)
    }

    fn is_request_initiated(&self) -> bool {
        self.conn.is_request_initiated()
    }

    fn initiate_response(&mut self) -> Result<(), Self::Error> {
        // 这个函数可能会返回一个timeout error
        if let Err(e) = self.conn.initiate_response() {
            error!("Error initiating response: {:?}", e);
            if e.code() == -ESP_ERR_HTTP_EAGAIN {
                // timeout error, need to build a new connection
                self.need_build_new_connection = true;
            }
            return Err(EspIOError::from(e));
        }
        Ok(())
    }

    fn is_response_initiated(&self) -> bool {
        self.conn.is_response_initiated()
    }

    fn split(&mut self) -> (&Self::Headers, &mut Self::Read) {
        let headers_ptr: *const Self = self as *const _;
        let headers = unsafe { headers_ptr.as_ref().unwrap() };
        (headers, self)
    }

    fn raw_connection(&mut self) -> Result<&mut Self::RawConnection, Self::Error> {
        Err(EspError::from_infallible::<ESP_FAIL>().into())
    }
}
