use poem::{listener::TcpListener, Result, Route, Server};
use poem_openapi::OpenApiService;

use serde::Deserialize;

mod error;
mod service;

#[derive(Debug, Deserialize)]
struct ServiceConfig {
    openwrt: service::OpenWrtServiceConfig,
    weather: service::WeatherServiceConfig,
}

#[derive(Debug, Deserialize)]
struct Config {
    bind_addr: String,
    redis_addr: String,
    service: ServiceConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let config = std::fs::read_to_string("config.yaml")?;
    let config = serde_yaml::from_str::<Config>(&config)?;
    let redis_cli = redis::Client::open(config.redis_addr)?;
    let service = OpenApiService::new(
        (
            service::PingService,
            service::PhotoService,
            service::OpenWrt::new(config.service.openwrt),
            service::WeatherService::new(config.service.weather, redis_cli.clone()),
        ),
        "Esp Clock Server Api",
        "1.0",
    )
    .server("/api");
    let swagger_ui = service.swagger_ui();
    Server::new(TcpListener::bind(config.bind_addr))
        .run(Route::new().nest("/api", service).nest("/ui", swagger_ui))
        .await?;
    Ok(())
}
