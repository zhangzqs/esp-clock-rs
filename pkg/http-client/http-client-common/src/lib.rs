use anyhow::Result;
use http::{Request, Response};

pub trait Client {
    fn send(&mut self, req: Request<&[u8]>) -> Result<Response<Vec<u8>>>;
    fn send_text(&mut self, req: Request<&str>) -> Result<Response<String>> {
        let req = req.map(|body| body.as_bytes());
        let res = self.send(req)?;
        let res = res.map(|body| String::from_utf8(body).unwrap());
        Ok(res)
    }
}

pub enum MyMethod {
    Get,
    Post,
}

pub struct MyRequest<'a, B> {
    pub method: MyMethod,
    pub uri: &'a str,
    pub headers: &'a [(&'a str, &'a str)],
    pub body: B,
}

pub struct MyResponse {
    status: u16,
}

pub trait MyClient {
    fn send<'a>(&mut self, req: MyRequest<'a, &[u8]>) -> Result<MyResponse<'a>>;
}
