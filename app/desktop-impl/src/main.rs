use std::time::Duration;

use app_core::{get_app_window, get_scheduler};
use slint::ComponentHandle;

mod http_client;
use http_client::HttpClient;

mod http_server;
use http_server::HttpServer;

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let app = get_app_window();
    let sche = get_scheduler();
    sche.register_node(HttpClient::new(4));
    sche.register_node(HttpServer::new());

    let sche_timer = slint::Timer::default();
    sche_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(20),
        move || {
            sche.schedule_once();
        },
    );

    if let Some(x) = app.upgrade() {
        x.run().unwrap();
    }
}
