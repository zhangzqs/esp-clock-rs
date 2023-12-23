use std::time::Duration;

use slint::Weak;
use time::{OffsetDateTime, UtcOffset};

use crate::{AppWindow, HomeTimeData};
pub struct HomeApp {
    app: Weak<AppWindow>,
    home_time_timer: slint::Timer,
}

impl HomeApp {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self {
            app: app.clone(),
            home_time_timer: Self::start_home_time_timer(app.clone()),
        }
    }

    fn start_home_time_timer(w: Weak<AppWindow>) -> slint::Timer {
        let t = slint::Timer::default();
        t.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(1),
            move || {
                let t = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
                if let Some(ui) = w.upgrade() {
                    ui.set_home_page_time(HomeTimeData {
                        day: t.day() as i32,
                        hour: t.hour() as i32,
                        minute: t.minute() as i32,
                        month: t.month() as i32,
                        second: t.second() as i32,
                        week: t.weekday().number_days_from_sunday() as i32,
                        year: t.year(),
                    });
                }
            },
        );
        t
    }
}
