use poem_openapi::{OpenApi, param::Query, payload::Json};
use serde_json::json;

pub struct Ping;

#[OpenApi]
impl Ping {
    #[oai(path = "/ping", method = "get")]
    async fn index(
        &self,
        name: Query<Option<String>>,
        age: Query<Option<u8>>,
    ) -> Json<serde_json::Value> { 
        Json(json!({
            "name": name.0,
            "age": age.0,
        }))
    }
}