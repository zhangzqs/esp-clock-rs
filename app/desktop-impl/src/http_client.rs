use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use proto::{
    HandleResult, HttpBody, HttpMessage, HttpRequest, HttpRequestMethod, HttpResponse, Message,
    MessageTo, MessageWithHeader, Node, NodeName,
};
use reqwest::blocking::ClientBuilder;

fn convert(method: HttpRequestMethod) -> reqwest::Method {
    use reqwest::Method;
    match method {
        HttpRequestMethod::Get => Method::GET,
    }
}

pub struct HttpClient {
    // 发送一个请求
    req_tx: mpsc::Sender<(u32, Arc<HttpRequest>)>,
    // 收到一个响应
    resp_rx: mpsc::Receiver<(u32, Arc<HttpResponse>)>,
    // 还在执行中的消息
    running_req: HashSet<u32>,
    // 已经就绪的响应
    ready_resp: HashMap<u32, Arc<HttpResponse>>,
}

impl HttpClient {
    pub fn new(threads: usize) -> Self {
        let (req_tx, req_rx) = mpsc::channel::<(u32, Arc<HttpRequest>)>();
        let (resp_tx, resp_rx) = mpsc::channel::<(u32, Arc<HttpResponse>)>();
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
                                Arc::new(HttpResponse {
                                    request: req.clone(),
                                    body: HttpBody::Bytes(content),
                                }),
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
            running_req: HashSet::new(),
            ready_resp: HashMap::new(),
        }
    }
}

impl Node for HttpClient {
    fn node_name(&self) -> NodeName {
        NodeName::HttpClient
    }

    fn handle_message(
        &mut self,
        _ctx: Rc<dyn proto::Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        match msg.body {
            Message::Http(HttpMessage::Request(req)) => {
                if self.running_req.contains(&msg.seq) {
                    // 若消息仍处于running态，继续返回 Pending，调度器后续继续轮询
                    match self.resp_rx.try_recv() {
                        Ok((seq, resp)) => {
                            // 当消息执行完成后，消息转换为ready态
                            self.running_req.remove(&msg.seq);
                            self.ready_resp.insert(seq, resp);
                            return HandleResult::Pending;
                        }
                        _ => {}
                    }
                    return HandleResult::Pending;
                } else if self.ready_resp.contains_key(&msg.seq) {
                    // 若消息结果为ready态，则返回Sucessful
                    return HandleResult::Successful(Message::Http(HttpMessage::Response(
                        self.ready_resp.remove(&msg.seq).unwrap(),
                    )));
                } else {
                    // 否则为新消息
                    self.req_tx.send((msg.seq, req)).unwrap();
                    self.running_req.insert(msg.seq);
                    return HandleResult::Pending;
                }
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
