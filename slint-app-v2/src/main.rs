mod app;
mod common;
mod scheduler;

use app::{HomeApp, WeatherApp};
use scheduler::Scheduler;

fn main() {
    let mut sche = Scheduler::new();
    sche.register_app(HomeApp::new());
    sche.register_app(WeatherApp::new());
    loop {
        sche.schedule_once();
    }
}
