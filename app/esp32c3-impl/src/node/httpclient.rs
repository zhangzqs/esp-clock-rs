use std::{
    collections::HashMap,
    io::Read,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use app_core::proto::*;
use embedded_io_adapters::std::ToStd;
use embedded_svc::http::client::Client;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use esp_idf_sys as _;
use libflate::gzip::{self};

pub struct HttpClientService {
    state: Arc<Mutex<HashMap<usize, (HttpRequest, Option<Message>)>>>,
}

impl HttpClientService {
    pub fn new() -> Self {
        let state: Arc<Mutex<HashMap<usize, (HttpRequest, Option<Message>)>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let state_ref = state.clone();
        thread::Builder::new()
            .stack_size(8 * 1024)
            .spawn(move || {
                loop {
                    for (_, (req, result)) in state_ref.lock().unwrap().iter_mut() {
                        let ret = (|| -> anyhow::Result<_> {
                            let conn = EspHttpConnection::new(&Configuration {
                                crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach), // https支持
                                ..Default::default()
                            })?;
                            let mut client = Client::wrap(conn);

                            let resp = client.get(&req.url)?.submit()?;
                            let resp_body = if let Some("gzip") = resp.header("content-encoding") {
                                gzip::Decoder::new(ToStd::new(resp))?
                                    .bytes()
                                    .try_collect::<Vec<_>>()?
                            } else {
                                ToStd::new(resp).bytes().try_collect::<Vec<_>>()?
                            };
                            Ok(HttpMessage::Response(HttpResponse {
                                body: HttpBody::Bytes(Bytes(resp_body)),
                            }))
                        })();
                        *result = Some(Message::Http(match ret {
                            Ok(x) => x,
                            Err(e) => HttpMessage::Error(HttpError::Other(format!("{e}"))),
                        }));
                    }

                    thread::sleep(Duration::from_millis(16));
                }
            })
            .unwrap();
        Self { state }
    }
}

impl Node for HttpClientService {
    fn node_name(&self) -> NodeName {
        NodeName::HttpClient
    }

    fn poll(&self, ctx: Rc<dyn Context>, seq: usize) {
        let mut l = self.state.lock().unwrap();
        if let (_, Some(result)) = &l[&seq] {
            // 消息有结果了
            ctx.async_ready(seq, result.clone());
            l.remove(&seq);
        }
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        if let Message::Http(HttpMessage::Request(req)) = msg.body {
            // 传送消息
            self.state.lock().unwrap().insert(msg.seq, (req, None));
            return HandleResult::Pending;
        }
        HandleResult::Discard
    }
}
