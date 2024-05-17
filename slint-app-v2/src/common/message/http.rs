use std::collections::HashMap;

pub enum HttpBody {
    Empty,
    Bytes(Vec<u8>),
}

pub enum HttpRequestMethod {
    Get,
}

pub struct HttpRequest {
    pub method: HttpRequestMethod,
    pub url: String,
    pub header: Option<HashMap<String, String>>,
    pub body: HttpBody,
}

pub struct HttpResponse {
    pub body: HttpBody,
}

#[derive(Debug, Clone)]
pub enum HttpMessage {
    Request(HttpRequest),
    Response(HttpResponse),
}
