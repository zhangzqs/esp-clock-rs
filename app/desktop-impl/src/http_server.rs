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

    fn handle(&self, ctx: Rc<dyn Context>) {
        match self.h.try_recv() {
            Ok(Some(mut raw_req)) => match serde_json::from_reader::<_, Req>(raw_req.as_reader()) {
                Ok(req_msg) => match req_msg.to {
                    MessageTo::Broadcast => {
                        ctx.broadcast_global(req_msg.body);
                    }
                    MessageTo::Topic(topic) => {
                        ctx.broadcast_topic(topic, req_msg.body);
                    }
                    MessageTo::Point(p) => {
                        ctx.async_call(
                            p,
                            req_msg.body,
                            Box::new(|r| {
                                let bs = serde_json::to_vec(&r).unwrap();
                                if let Err(e) = raw_req.respond(Response::from_data(bs)) {
                                    error!("http server write err: {e:?}");
                                }
                            }),
                        );
                    }
                },
                Err(e) => {
                    if let Err(e) =
                        raw_req.respond(Response::from_string(e.to_string()).with_status_code(400))
                    {
                        error!("http server write err: {e:?}");
                    }
                }
            },
            _ => {}
        }
    }
}

impl Node for HttpServer {
    fn node_name(&self) -> NodeName {
        NodeName::Other("HttpServer".into())
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(LifecycleMessage::Init) => ctx.subscribe_topic(TopicName::Scheduler),
            Message::Empty => self.handle(ctx.clone()),
            _ => {}
        }
        HandleResult::Discard
    }
}
