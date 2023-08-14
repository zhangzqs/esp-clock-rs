use std::{thread, time::Duration};
use log::info;
use std::sync::mpsc;
use time::{OffsetDateTime, UtcOffset};

slint::include_modules!();

pub struct MyApp {
    app_window: AppWindow,
    timer: slint::Timer,
    page_timer: slint::Timer,
}


impl MyApp {
    pub fn go_to_next_page(&self) {
        info!("go_to_next_page");
        self.page_timer.restart();
        
        let u = self.app_window.as_weak();
        if let Some(ui) = u.upgrade() {
            let s = ui.get_page_size();
            let p = ui.get_page_id();
            ui.set_page_id((p + 1) % s);
        }
    }

    pub fn go_to_prev_page(&self) {
        info!("go_to_prev_page");
        self.page_timer.restart();

        let u = self.app_window.as_weak();
        if let Some(ui) = u.upgrade() {
            let s = ui.get_page_size();
            let p = ui.get_page_id();
            ui.set_page_id((p + s - 1) % s);
        }
    }

    pub fn go_to_home_page(&self) {
        info!("go_to_home_page");
        self.page_timer.restart();

        let u = self.app_window.as_weak();
        if let Some(ui) = u.upgrade() {
            ui.set_page_id(0);
        }
    }

    pub fn new() -> MyApp {
        let app_window = AppWindow::new().expect("Failed to create AppWindow");
        MyApp {
            timer: {
                let t = slint::Timer::default();
                let u = app_window.as_weak();
                t.start(slint::TimerMode::Repeated, Duration::from_secs(1), move || {
                    let t = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
                    let dt = DateTime {
                        day: t.day() as i32,
                        hour: t.hour()  as i32,
                        minute: t.minute() as i32,
                        month: t.month() as i32,
                        second: t.second() as i32,
                        week: t.weekday().number_days_from_sunday() as i32,
                        year: t.year() as i32,
                    };
                    if let Some(ui) = u.upgrade() {
                        ui.set_date_time(dt);
                    }
                });
                t
            },
            page_timer: {
                let t = slint::Timer::default();
                let u = app_window.as_weak();
                t.start(slint::TimerMode::Repeated, Duration::from_secs(5), move || {
                    if let Some(ui) = u.upgrade() {
                        let s = ui.get_page_size();
                        let p = ui.get_page_id();
                        ui.set_page_id((p + 1) % s);
                    }
                });
                t
            },
            app_window,
        }
    }
}