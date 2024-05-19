use std::time::Duration;

use app_core::{get_app_window, get_schedular};
use slint::ComponentHandle;

fn main() {
    let app = get_app_window();
    let mut sche = get_schedular();
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
