use std::rc::Rc;

use app_core::proto::{
    Context, HandleResult, HttpBody, HttpError, HttpMessage, HttpRequestMethod, HttpResponse,
    Message, MessageTo, MessageWithHeader, Node, NodeName,
};

fn convert(method: HttpRequestMethod) -> reqwest::Method {
    use reqwest::Method;
    match method {
        HttpRequestMethod::Get => Method::GET,
    }
}

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for HttpClient {
    fn node_name(&self) -> NodeName {
        NodeName::HttpClient
    }

    fn handle_message(
        &self,
        ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        match msg.body {
            Message::Http(HttpMessage::Request(req)) => {
                if let Some(x) = msg.ready_result {
                    return HandleResult::Finish(x);
                }
                if msg.is_pending {
                    // 若消息仍处于running态，继续返回 Pending，调度器后续继续轮询
                    return HandleResult::Pending;
                }
                // 否则为新消息
                let req = req.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let x = async {
                        let req = reqwest::Client::new()
                            .request(convert(req.method.clone()), req.url.clone())
                            .build()
                            .map_err(|x| HttpError::Other(x.to_string()))?;
                        let resp = reqwest::Client::new()
                            .execute(req)
                            .await
                            .map_err(|x| HttpError::Other(x.to_string()))?;
                        let body = resp
                            .bytes()
                            .await
                            .map_err(|x| HttpError::Other(x.to_string()))?
                            .to_vec();
                        Ok(HttpResponse {
                            body: HttpBody::Bytes(body),
                        })
                    }
                    .await;
                    ctx.async_ready(
                        msg.seq,
                        Message::Http(match x {
                            Ok(x) => HttpMessage::Response(x),
                            Err(e) => HttpMessage::Error(e),
                        }),
                    );
                });
                return HandleResult::Pending;
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
