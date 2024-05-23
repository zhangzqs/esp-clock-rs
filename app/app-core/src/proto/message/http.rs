use serde::de;

#[derive(Debug, Clone)]
pub enum HttpBody {
    Bytes(Vec<u8>),
    Stream,
}

impl HttpBody {
    pub fn deserialize_by_json<'a, T>(&'a self) -> serde_json::Result<T>
    where
        T: de::Deserialize<'a>,
    {
        match self {
            HttpBody::Bytes(bs) => serde_json::from_slice::<T>(bs),
            HttpBody::Stream => {
                unimplemented!("not implement");
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum HttpRequestMethod {
    Get,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpRequestMethod,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub body: HttpBody,
}

#[derive(Debug, Clone)]
pub enum HttpError {
    Timeout,
    Other(String),
}

#[derive(Debug, Clone)]
pub enum HttpMessage {
    Error(HttpError),
    Request(HttpRequest),
    Response(HttpResponse),
}
