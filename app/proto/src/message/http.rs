use std::{collections::HashMap, rc::Rc, sync::Arc};

#[derive(Debug, Clone)]
pub enum HttpBody {
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
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub request: Arc<HttpRequest>,
    pub body: HttpBody,
}

#[derive(Debug, Clone)]
pub enum HttpMessage {
    Request(Arc<HttpRequest>),
    Response(Arc<HttpResponse>),
}
