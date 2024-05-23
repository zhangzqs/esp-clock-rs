use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use app_core::proto::{
    Context, HandleResult, HttpBody, HttpMessage, HttpRequest, HttpRequestMethod, HttpResponse,
    Message, MessageTo, MessageWithHeader, Node, NodeName,
};
use reqwest::blocking::ClientBuilder;

fn convert(method: HttpRequestMethod) -> reqwest::Method {
    use reqwest::Method;
    match method {
        HttpRequestMethod::Get => Method::GET,
    }
}

struct State {
    // 已经就绪的响应
    ready_resp: HashMap<u32, HandleResult>,
}

pub struct HttpClient {
    // 发送一个请求
    req_tx: mpsc::Sender<(u32, HttpRequest)>,
    // 收到一个响应
    resp_rx: mpsc::Receiver<(u32, HandleResult)>,
    state: RefCell<State>,
}

impl HttpClient {
    pub fn new(threads: usize) -> Self {
        let (req_tx, req_rx) = mpsc::channel::<(u32, HttpRequest)>();
        let (resp_tx, resp_rx) = mpsc::channel();
        let client = ClientBuilder::new().build().unwrap();

        let req_rx = Arc::new(Mutex::new(req_rx));
        for _ in 0..threads {
            let resp_tx = resp_tx.clone();
            let req_rx = req_rx.clone();
            let client = client.clone();
            thread::spawn(move || loop {
                match req_rx.lock().unwrap().try_recv() {
                    Ok((seq, req)) => {
                        let resp = client
                            .execute(
                                client
                                    .request(convert(req.method.clone()), req.url.clone())
                                    .build()
                                    .unwrap(),
                            )
                            .unwrap();
                        let content = resp.bytes().unwrap().to_vec();
                        resp_tx
                            .send((
                                seq,
                                HandleResult::Finish(Message::Http(HttpMessage::Response(
                                    HttpResponse {
                                        body: HttpBody::Bytes(content),
                                    },
                                ))),
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
            });
        }
        Self {
            req_tx,
            resp_rx,
            state: RefCell::new(State {
                ready_resp: HashMap::new(),
            }),
        }
    }
}

impl Node for HttpClient {
    fn node_name(&self) -> NodeName {
        NodeName::HttpClient
    }

    fn handle_message(
        &self,
        _ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        match msg.body {
            Message::Http(HttpMessage::Request(req)) => {
                let mut state = self.state.borrow_mut();
                if state.ready_resp.contains_key(&msg.seq) {
                    // 若消息结果为ready态，则返回Sucessful
                    return state.ready_resp.remove(&msg.seq).unwrap();
                }

                if msg.is_pending {
                    // 若消息仍处于pending态，且无返回结果，继续返回 Pending，调度器后续继续轮询
                    match self.resp_rx.try_recv() {
                        Ok((seq, resp)) => {
                            // 当消息执行完成后，消息转换为ready态
                            state.ready_resp.insert(seq, resp);
                            return HandleResult::Pending;
                        }
                        _ => {}
                    }
                    return HandleResult::Pending;
                }

                // 否则为新消息
                self.req_tx.send((msg.seq, req)).unwrap();
                return HandleResult::Pending;
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
