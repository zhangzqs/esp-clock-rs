use node::{BootPage, HomePage, MenuPage, RouterService, TouchOneButtonAdapterService, WeatherPage};
use scheduler::Scheduler;

mod node;
mod scheduler;
mod ui;
mod adapter;

pub use ui::get_app_window;

pub fn get_schedular() -> Scheduler {
    let app = get_app_window();

    let mut sche = Scheduler::new();
    sche.register_node(HomePage::new(app.clone()));
    sche.register_node(WeatherPage::new());
    sche.register_node(MenuPage::new(app.clone()));
    sche.register_node(BootPage::new(app.clone()));

    sche.register_node(RouterService::new(app.clone()));
    sche.register_node(TouchOneButtonAdapterService::new(app.clone()));

    sche
}
