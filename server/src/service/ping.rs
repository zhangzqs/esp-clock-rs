use poem_openapi::{
    payload::{PlainText},
    OpenApi,
};


pub struct PingService;

#[OpenApi]
impl PingService {
    #[oai(path = "/ping", method = "get")]
    async fn index(&self) -> PlainText<String> {
        PlainText("pong".to_string())
    }
}
