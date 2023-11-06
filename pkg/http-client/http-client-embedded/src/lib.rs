use anyhow::Result;
use embedded_svc::{http::client, utils::io};
use http::{Request, Response};

pub struct HttpClient<C: client::Connection> {
    client: client::Client<C>,
}

impl<C: client::Connection> HttpClient<C> {
    pub fn new(client: client::Client<C>) -> Self {
        Self { client }
    }
}

impl<C: client::Connection> http_client_common::Client for HttpClient<C> {
    fn send(&mut self, req: Request<&[u8]>) -> Result<Response<Vec<u8>>> {
        let uri = req.uri().to_string();
        let headers = req
            .headers()
            .iter()
            .map(|(k, v)| (k, String::from_utf8(v.as_bytes().to_vec()).unwrap()))
            .collect::<Vec<_>>();
        let headers = headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect::<Vec<_>>();
        let h = headers.as_slice();
        let method = match *req.method() {
            http::Method::GET => client::Method::Get,
            http::Method::POST => client::Method::Post,
            http::Method::PUT => client::Method::Put,
            http::Method::DELETE => client::Method::Delete,
            http::Method::HEAD => client::Method::Head,
            http::Method::OPTIONS => client::Method::Options,
            http::Method::CONNECT => client::Method::Connect,
            http::Method::PATCH => client::Method::Patch,
            http::Method::TRACE => client::Method::Trace,
            _ => unreachable!(),
        };
        let mut request = self.client.request(method, &uri, h).unwrap();
        request.write(req.body()).unwrap();
        let mut response = request.submit().unwrap();
        let (a, b) = response.split();

        let a = Response::builder()
            .status(response.status())
            .body({
                let mut ret = Vec::<u8>::new();
                let mut buf = [0u8; 512];
                while let Ok(l) = response.read(&mut buf) {
                    if l == 0 {
                        break;
                    }
                    ret.extend_from_slice(&buf[..l]);
                }
                ret
            })
            .unwrap();
        Ok(a)
    }
}
