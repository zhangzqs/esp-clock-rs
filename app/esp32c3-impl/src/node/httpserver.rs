use std::{
    cell::RefCell,
    io::Write,
    rc::Rc,
    sync::mpsc::{self, Receiver, SyncSender},
};

use app_core::proto::*;
use embedded_io_adapters::std::ToStd;
use esp_idf_hal::io::Write as _;
use esp_idf_svc::http::{
    server::{Configuration, EspHttpServer},
    Method,
};
use esp_idf_sys::{self as _, EspError};
use serde::{Deserialize, Serialize};
use serde_json::json;

// static INDEX_HTML: &[u8] = include_bytes!("../../../../vue-console/dist/index.html");

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HttpMessage {
    to: MessageTo,
    body: Message,
    // 是否为同步消息？
    #[serde(default)]
    is_sync: bool,
}

struct State {
    _server: EspHttpServer<'static>,
    req_rx: Receiver<HttpMessage>,
    resp_tx: SyncSender<HandleResult>,
}

impl State {
    fn new() -> Self {
        let (req_tx, req_rx) = mpsc::sync_channel(1);
        let (resp_tx, resp_rx) = mpsc::sync_channel(1);
        let mut server = EspHttpServer::new(&Configuration {
            ..Default::default()
        })
        .unwrap();
        server
            .fn_handler("/", Method::Get, |req| {
                req.into_ok_response()?.write_all(b"not found index.html")?;
                Ok(())
            })
            .unwrap()
            .fn_handler("/", Method::Post, move |mut req| {
                let resp_body: anyhow::Result<HandleResult> = (|| {
                    let req_body = serde_json::from_reader::<_, HttpMessage>(ToStd::new(&mut req))?;
                    req_tx.send(req_body)?;
                    Ok(resp_rx.recv()?)
                })();
                match resp_body {
                    Ok(x) => {
                        let resp = req.into_ok_response()?;
                        serde_json::to_writer(ToStd::new(resp), &x)?;
                    }
                    Err(e) => {
                        let resp = req.into_status_response(500)?;
                        serde_json::to_writer(
                            ToStd::new(resp),
                            &json!({
                                "error": format!("{e:?}"),
                            }),
                        )?;
                    }
                }
                Ok(())
            })
            .unwrap();
        Self {
            req_rx,
            resp_tx,
            _server: server,
        }
    }
}

pub struct HttpServerService {
    state: RefCell<Option<State>>,
}

impl HttpServerService {
    pub fn new() -> Self {
        Self {
            state: RefCell::new(None),
        }
    }

    fn handle_request(&self, ctx: Rc<dyn Context>) {
        if let Some(s) = &*self.state.borrow() {
            if let Ok(x) = s.req_rx.try_recv() {
                let tx = s.resp_tx.clone();
                match x.to {
                    MessageTo::Broadcast => {
                        ctx.broadcast_global(x.body);
                        tx.send(HandleResult::Finish(Message::Empty)).unwrap();
                    }
                    MessageTo::Topic(topic) => {
                        ctx.broadcast_topic(topic, x.body);
                        tx.send(HandleResult::Finish(Message::Empty)).unwrap();
                    }
                    MessageTo::Point(node) => {
                        if x.is_sync {
                            let m = ctx.sync_call(node, x.body);
                            tx.send(m).unwrap();
                        } else {
                            ctx.async_call(
                                node,
                                x.body,
                                Box::new(move |m| {
                                    tx.send(m).unwrap();
                                }),
                            );
                        }
                    }
                }
            }
        }
    }
}

impl Node for HttpServerService {
    fn node_name(&self) -> NodeName {
        NodeName::Other("EspHttpServer".into())
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::WiFi(WiFiMessage::ConnectedBoardcast) => {
                ctx.subscribe_topic(TopicName::Scheduler);
                self.state.borrow_mut().replace(State::new());
                return HandleResult::Finish(Message::Empty);
            }
            _ => {
                self.handle_request(ctx.clone());
            }
        }
        HandleResult::Discard
    }
}
