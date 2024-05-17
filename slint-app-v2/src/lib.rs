use app::{BootPageApp, HomePageApp, MenuPageApp, RouterApp, TouchOneButtonApp, WeatherPageApp};
use scheduler::Scheduler;

mod app;
mod common;
mod scheduler;
mod ui;

pub use ui::get_app_window;

pub fn get_schedular() -> Scheduler {
    let app = get_app_window();

    let mut sche = Scheduler::new();
    sche.register_app(HomePageApp::new(app.clone()));
    sche.register_app(WeatherPageApp::new());
    sche.register_app(MenuPageApp::new(app.clone()));
    sche.register_app(BootPageApp::new(app.clone()));

    sche.register_app(RouterApp::new(app.clone()));
    sche.register_app(TouchOneButtonApp::new(app.clone()));

    sche
}
