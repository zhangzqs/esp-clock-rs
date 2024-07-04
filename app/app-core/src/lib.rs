use std::rc::Rc;

use node::*;

mod adapter;
mod node;
mod scheduler;
mod ui;

pub use proto;
pub use proto::storage;
pub use scheduler::Scheduler;
pub use ui::get_app_window;

static mut SCHEDULER: Option<Rc<Scheduler>> = None;

pub fn get_scheduler() -> Rc<Scheduler> {
    unsafe {
        SCHEDULER
            .get_or_insert_with(|| {
                let s = Scheduler::new();
                register_default_nodes(&s);
                Rc::new(s)
            })
            .clone()
    }
}

fn register_default_nodes(sche: &Scheduler) {
    sche.register_node(HomePage::new());
    sche.register_node(WeatherPage::new());
    sche.register_node(MenuPage::new());
    sche.register_node(BootPage::new());
    sche.register_node(AlertDialog::new());
    sche.register_node(MusicPage::new());

    sche.register_node(RouterService::new());
    sche.register_node(TouchOneButtonAdapterService::new());
    sche.register_node(WeatherService::new());
    sche.register_node(MockStorageService::new());
    sche.register_node(MockSystemService {});
    sche.register_node(TimerService::new());

    sche.register_node(MockWiFiService::new());
    sche.register_node(MidiPlayerService::new());
    sche.register_node(CanvasView::new());
    sche.register_node(UserAlarmService::new());
}
