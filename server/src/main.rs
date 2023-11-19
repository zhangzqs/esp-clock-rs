use poem::{listener::TcpListener, Result, Route, Server};
use poem_openapi::OpenApiService;
use serde::Deserialize;

mod service;

#[derive(Debug, Deserialize)]
struct ServiceConfig {
    openwrt: service::OpenWrtServiceConfig,
}

#[derive(Debug, Deserialize)]
struct Config {
    bind_addr: String,
    service: ServiceConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = std::fs::read_to_string("config.yaml")?;
    let config = serde_yaml::from_str::<Config>(&config)?;
    let service = OpenApiService::new(
        (
            service::Ping,
            service::OpenWrt::new(config.service.openwrt),
            service::Photo,
        ),
        "HelloWorld",
        "1.0",
    )
    .server("http://localhost:3000/api");
    let swagger_ui = service.swagger_ui();
    Server::new(TcpListener::bind(config.bind_addr))
        .run(Route::new().nest("/api", service).nest("/ui", swagger_ui))
        .await?;
    Ok(())
}
