use std::rc::Rc;

use crate::{Context, Message, NodeName};

use super::AsyncResultCallback;
use crate::message::{HttpError, HttpMessage, HttpRequest, HttpResponse};

#[derive(Clone)]
pub struct HttpClient(pub Rc<dyn Context>);

impl HttpClient {
    pub fn request(
        &self,
        request: HttpRequest,
        callback: AsyncResultCallback<HttpResponse, HttpError>,
    ) {
        self.0.async_call(
            NodeName::HttpClient,
            Message::Http(HttpMessage::Request(request)),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Http(HttpMessage::Response(resp)) => Ok(resp),
                    Message::Http(HttpMessage::Error(e)) => Err(e),
                    m => panic!("unexpected HandleResult {:?}", m),
                });
            }),
        )
    }
}
