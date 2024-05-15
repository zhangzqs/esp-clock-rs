use app::{BootPageApp, HomePageApp, MenuPageApp, RouterApp, TouchOneButtonApp, WeatherPageApp};
use scheduler::Scheduler;
use slint::{ComponentHandle, TimerMode};
use std::time::Duration;
use ui::AppWindow;

mod app;
mod common;
mod scheduler;
mod ui;

fn main() {
    let app = AppWindow::new().unwrap();

    let mut sche = Scheduler::new();
    sche.register_app(HomePageApp::new(app.as_weak()));
    sche.register_app(WeatherPageApp::new());
    sche.register_app(MenuPageApp::new(app.as_weak()));
    sche.register_app(BootPageApp::new(app.as_weak()));

    sche.register_app(RouterApp::new(app.as_weak()));
    sche.register_app(TouchOneButtonApp::new(app.as_weak()));

    let t = slint::Timer::default();
    t.start(TimerMode::Repeated, Duration::from_millis(20), move || {
        sche.schedule_once();
    });
    app.run().unwrap()
}
