use std::{
    cell::RefCell, collections::HashMap, io::Read, rc::Rc, sync::mpsc, thread, time::Duration,
};

use app_core::proto::*;
use embedded_io_adapters::std::ToStd;
use embedded_svc::http::client::Client;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use esp_idf_sys as _;

struct State {
    // 已经就绪的响应
    ready_resp: HashMap<usize, Message>,
}

pub struct HttpClientService {
    // 发送一个请求
    req_tx: mpsc::Sender<(usize, HttpRequest)>,
    // 收到一个响应
    resp_rx: mpsc::Receiver<(usize, Message)>,
    state: RefCell<State>,
}

impl HttpClientService {
    pub fn new() -> Self {
        let (req_tx, req_rx) = mpsc::channel::<(usize, HttpRequest)>();
        let (resp_tx, resp_rx) = mpsc::channel();

        thread::spawn(move || {
            let conn = EspHttpConnection::new(&Configuration::default()).unwrap();
            let mut client = Client::wrap(conn);
            loop {
                match req_rx.try_recv() {
                    Ok((seq, req)) => {
                        let req = client.get(&req.url).unwrap().submit().unwrap();
                        let resp_std = ToStd::new(req);
                        let resp_body = resp_std.bytes().map(|x| x.unwrap()).collect::<Vec<_>>();
                        resp_tx
                            .send((
                                seq,
                                Message::Http(HttpMessage::Response(HttpResponse {
                                    body: HttpBody::Bytes(resp_body),
                                })),
                            ))
                            .unwrap();
                    }
                    Err(e) => match e {
                        mpsc::TryRecvError::Empty => {
                            thread::sleep(Duration::from_millis(10));
                        }
                        mpsc::TryRecvError::Disconnected => {
                            return;
                        }
                    },
                }
            }
        });
        Self {
            req_tx,
            resp_rx,
            state: RefCell::new(State {
                ready_resp: HashMap::new(),
            }),
        }
    }
}

impl Node for HttpClientService {
    fn node_name(&self) -> NodeName {
        NodeName::HttpClient
    }

    fn poll(&self, ctx: Rc<dyn Context>, seq: usize) {
        let mut state = self.state.borrow_mut();
        if let Ok((seq, resp)) = self.resp_rx.try_recv() {
            // 当消息执行完成后，消息转换为ready态
            state.ready_resp.insert(seq, resp);
        }
        if state.ready_resp.contains_key(&seq) {
            // 若消息结果为ready态，则返回Sucessful
            let ret = state.ready_resp.remove(&seq).unwrap();
            ctx.async_ready(seq, ret);
        }
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        if let Message::Http(HttpMessage::Request(req)) = msg.body {
            // 传送消息
            self.req_tx.send((msg.seq, req)).unwrap();
            return HandleResult::Pending;
        }
        HandleResult::Discard
    }
}
