use poem::{listener::TcpListener, Route, Server, Result};
use poem_openapi::{param::Query, payload::Json, OpenApi, OpenApiService, Object};
use serde::Serialize;

struct Api;

#[derive(Object, Serialize)]
struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    age: Option<u8>,
}

#[OpenApi]
impl Api {
    #[oai(path = "/hello", method = "get")]
    async fn index(&self, name: Query<Option<String>>, age: Query<Option<u8>>) -> Json<User> {
        Json(User {
            name: name.0,
            age: age.0,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_service = OpenApiService::new(Api, "HelloWorld", "1.0").server("http://localhost:3000/api");
    let ui = api_service.swagger_ui();
    Server::new(TcpListener::bind("127.0.0.1:3000"))
    .run(Route::new().nest("/api", api_service).nest("/", ui))
    .await?;
    Ok(())
}
