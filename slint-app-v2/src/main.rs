mod app;
mod common;
mod scheduler;

use std::time::Duration;

use app::{AppWindow, MenuPageApp, RouterApp};
use app::{HomePageApp, WeatherPageApp};
use scheduler::Scheduler;
use slint::{ComponentHandle, TimerMode};

fn main() {
    let app = AppWindow::new().unwrap();

    let mut sche = Scheduler::new();
    sche.register_app(HomePageApp::new(app.as_weak()));
    sche.register_app(WeatherPageApp::new());
    sche.register_app(RouterApp::new(app.as_weak()));
    sche.register_app(MenuPageApp::new(app.as_weak()));
    
    let t = slint::Timer::default();
    t.start(TimerMode::Repeated, Duration::from_millis(100), move || {
        sche.schedule_once();
    });
    app.run().unwrap()
}
