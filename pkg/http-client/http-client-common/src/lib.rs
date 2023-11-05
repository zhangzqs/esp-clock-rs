use anyhow::Result;
use http::{Request, Response};

pub trait Client {
    fn send(&self, req: Request<&[u8]>) -> Result<Response<Vec<u8>>>;
    fn send_text(&self, req: Request<&str>) -> Result<Response<String>> {
        let req = req.map(|body| body.as_bytes());
        let res = self.send(req)?;
        let res = res.map(|body| String::from_utf8(body).unwrap());
        Ok(res)
    }
}
