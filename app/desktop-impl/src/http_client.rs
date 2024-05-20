use std::{
    rc::Rc,
    sync::{mpsc, Arc},
    thread,
};

use proto::{
    HandleResult, HttpMessage, HttpRequest, HttpRequestMethod, LifecycleMessage, Message,
    MessageTo, Node, NodeName,
};
use reqwest::blocking::{Client, ClientBuilder};

fn convert(method: HttpRequestMethod) -> reqwest::Method {
    use reqwest::Method;
    match method {
        HttpRequestMethod::Get => Method::GET,
    }
}

struct HttpClient {
    tx: mpsc::Sender<Arc<HttpRequest>>,
}

impl HttpClient {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Arc<HttpRequest>>();
        thread::spawn(move || {
            let client = ClientBuilder::new().build().unwrap();

            for req in rx.iter() {
                let req = client
                    .request(convert(req.method.clone()), req.url.clone())
                    .build()
                    .unwrap();
                let resp = client.execute(req).unwrap();
                resp.bytes();
                
            }
        });
        Self { tx }
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
        msg: Message,
    ) -> HandleResult {
        match msg {
            Message::Http(HttpMessage::Request(req)) => {
                self.tx.send(req).unwrap();
            }
            _ => {}
        }
        return HandleResult::Discard;
    }
}
