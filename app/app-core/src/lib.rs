use node::{
    BootPage, HomePage, MenuPage, RouterService, TimestampClientService,
    TouchOneButtonAdapterService, WeatherPage,
};
pub use scheduler::{Scheduler, Platform};

mod adapter;
mod node;
mod scheduler;
mod ui;

use slint::ComponentHandle;
pub use ui::get_app_window;

pub fn get_scheduler() -> Scheduler {
    let mut sche = Scheduler::new();
    register_default_nodes(&mut sche);
    sche
}

pub fn register_default_nodes(sche: &mut Scheduler) {
    let app = get_app_window();
    sche.register_node(HomePage::new(app.clone()));
    sche.register_node(WeatherPage::new());
    sche.register_node(MenuPage::new(app.clone()));
    sche.register_node(BootPage::new(app.clone()));

    sche.register_node(RouterService::new(app.clone()));
    sche.register_node(TouchOneButtonAdapterService::new(app.clone()));
    sche.register_node(TimestampClientService {});
}
