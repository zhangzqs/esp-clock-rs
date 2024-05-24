use std::rc::Rc;

use app_core::proto::*;

use log::error;
use serde::{Deserialize, Serialize};
use tiny_http::Response;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Req {
    to: MessageTo,
    body: Message,
}

pub struct HttpServer {
    h: tiny_http::Server,
}

impl HttpServer {
    pub fn new() -> Self {
        let h = tiny_http::Server::http("127.0.0.1:38080").unwrap();
        Self { h }
    }
}

impl Node for HttpServer {
    fn node_name(&self) -> NodeName {
        NodeName::Other("HttpServer".into())
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, _msg: MessageWithHeader) -> HandleResult {
        match self.h.try_recv() {
            Ok(Some(mut raw_req)) => match serde_json::from_reader::<_, Req>(raw_req.as_reader()) {
                Ok(req_msg) => match req_msg.to {
                    MessageTo::Broadcast => {
                        ctx.boardcast(req_msg.body);
                    }
                    MessageTo::Point(p) => {
                        ctx.async_call(
                            p,
                            req_msg.body,
                            Box::new(|r| {
                                let bs = serde_json::to_vec(&r).unwrap();
                                raw_req.respond(Response::from_data(bs)).unwrap();
                            }),
                        );
                    }
                },
                Err(e) => {
                    error!("http server err: {e:?}");
                }
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
