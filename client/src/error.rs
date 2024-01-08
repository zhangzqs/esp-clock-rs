use std::fmt::{Debug, Display};

use embedded_svc::http::client::Connection;

#[derive(thiserror::Error, Debug)]
pub enum ClientError<C, E>
where
    C: Connection<Error = E> + Debug,
    E: Display,
{
    #[error("http error: {0}")]
    Http(C::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    ReadExactError(#[from] embedded_io::ReadExactError<E>),
    #[error("unknown content length, header: {0:?}")]
    UnknownContentLength(Option<String>),
    #[error("buffer too small, content length: {content_length}, buffer length: {buffer_length}")]
    BufferTooSmall {
        content_length: usize,
        buffer_length: usize,
    },
}
