use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{self, Receiver, SyncSender},
};

use app_core::proto::*;
use embedded_io_adapters::std::ToStd;
use esp_idf_svc::http::{
    server::{Configuration, EspHttpServer},
    Method,
};
use esp_idf_sys as _;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HttpMessage {
    to: MessageTo,
    body: Message,
}

struct State {
    server: EspHttpServer<'static>,
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
            .fn_handler("/", Method::Post, move |mut req| {
                let req_body = serde_json::from_reader::<_, HttpMessage>(ToStd::new(&mut req))?;
                req_tx.send(req_body)?;
                let resp_body: HandleResult = resp_rx.recv()?;
                let resp = req.into_ok_response()?;
                serde_json::to_writer(ToStd::new(resp), &resp_body)?;
                Ok(())
            })
            .unwrap();
        Self {
            req_rx,
            resp_tx,
            server,
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
}

impl Node for HttpServerService {
    fn node_name(&self) -> NodeName {
        NodeName::Other("EspHttpServer".into())
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::WiFi(WiFiMessage::ConnectedBoardcast) => {
                self.state.borrow_mut().replace(State::new());
            }
            _ => {
                if let Some(s) = &*self.state.borrow() {
                    if let Ok(x) = s.req_rx.try_recv() {
                        let tx = s.resp_tx.clone();
                        match x.to {
                            MessageTo::Broadcast => {
                                ctx.boardcast(x.body);
                                tx.send(HandleResult::Finish(Message::Empty)).unwrap();
                            }
                            MessageTo::Point(node) => {
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
        HandleResult::Discard
    }
}
