use std::{cell::RefCell, collections::HashMap, rc::Rc};

use app_core::proto::{
    Context, HandleResult, HttpBody, HttpMessage, HttpRequestMethod, HttpResponse, Message,
    MessageTo, MessageWithHeader, Node, NodeName,
};

fn convert(method: HttpRequestMethod) -> reqwest::Method {
    use reqwest::Method;
    match method {
        HttpRequestMethod::Get => Method::GET,
    }
}

pub struct HttpClient {
    // 已经就绪的响应
    ready_resp: Rc<RefCell<HashMap<u32, HandleResult>>>,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            ready_resp: Rc::new(RefCell::new(HashMap::new())),
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
                if self.ready_resp.borrow().contains_key(&msg.seq) {
                    // 若消息结果为ready态，则返回Sucessful
                    return self.ready_resp.borrow_mut().remove(&msg.seq).unwrap();
                }
                if msg.is_pending {
                    // 若消息仍处于running态，继续返回 Pending，调度器后续继续轮询
                    return HandleResult::Pending;
                }
                // 否则为新消息
                let req = req.clone();
                let ready_resp = self.ready_resp.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let resp = reqwest::Client::new()
                        .execute(
                            reqwest::Client::new()
                                .request(convert(req.method.clone()), req.url.clone())
                                .build()
                                .unwrap(),
                        )
                        .await
                        .unwrap();
                    let body = resp.bytes().await.unwrap().to_vec();
                    ready_resp.borrow_mut().insert(
                        msg.seq,
                        HandleResult::Finish(Message::Http(HttpMessage::Response(HttpResponse {
                            body: HttpBody::Bytes(body),
                        }))),
                    );
                });
                return HandleResult::Pending;
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
