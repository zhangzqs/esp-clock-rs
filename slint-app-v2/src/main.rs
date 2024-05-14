mod app;
mod common;
mod scheduler;

use std::time::Duration;

use app::AppWindow;
use app::{HomeApp, WeatherApp};
use scheduler::Scheduler;
use slint::{ComponentHandle, TimerMode};

fn main() {
    let app = AppWindow::new().unwrap();

    let mut sche = Scheduler::new();
    sche.register_app(HomeApp::new(app.as_weak()));
    sche.register_app(WeatherApp::new());
    let t = slint::Timer::default();
    t.start(TimerMode::Repeated, Duration::from_millis(100), move || {
        sche.schedule_once();
    });
    app.run().unwrap()
}
