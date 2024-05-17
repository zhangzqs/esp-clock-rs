use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
pub enum HttpBody {
    Empty,
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum HttpRequestMethod {
    Get,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpRequestMethod,
    pub url: String,
    pub header: Option<HashMap<String, String>>,
    pub body: HttpBody,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub body: HttpBody,
}

#[derive(Debug, Clone)]
pub enum HttpMessage {
    Request(Rc<HttpRequest>),
    Response(Rc<HttpResponse>),
}
