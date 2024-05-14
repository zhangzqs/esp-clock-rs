use std::{rc::Rc, time::Duration};

use ::time::{OffsetDateTime, UtcOffset};
use slint::{ComponentHandle, Weak};

use super::{AppWindow, HomeViewModel, TimeData, WeatherData};
use crate::common::{App, AppName, Context, HomeMessage, LifecycleMessage, Message, MessageTo};

pub struct HomePageApp {
    app: Weak<AppWindow>,
    time_update_timer: Option<slint::Timer>,
    weather_update_timer: Option<slint::Timer>,
}

impl HomePageApp {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self {
            app,
            time_update_timer: None,
            weather_update_timer: None,
        }
    }
}

impl HomePageApp {
    fn init(&mut self, ctx: Rc<Box<dyn Context>>) {
        let app = self.app.clone();
        self.time_update_timer
            .get_or_insert(slint::Timer::default())
            .start(
                slint::TimerMode::Repeated,
                Duration::from_secs(1),
                move || {
                    if let Some(ui) = app.upgrade() {
                        let home_app = ui.global::<HomeViewModel>();
                        let t = OffsetDateTime::now_utc()
                            .to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
                        home_app.set_time(TimeData {
                            day: t.day() as i32,
                            hour: t.hour() as i32,
                            minute: t.minute() as i32,
                            month: t.month() as i32,
                            second: t.second() as i32,
                            week: t.weekday().number_days_from_sunday() as i32,
                            year: t.year(),
                        })
                    }
                },
            );
        self.weather_update_timer
            .get_or_insert(slint::Timer::default())
            .start(
                slint::TimerMode::Repeated,
                Duration::from_secs(60),
                move || {
                    ctx.send_message(
                        MessageTo::App(AppName::WeatherClient),
                        Message::HomePage(HomeMessage::RequestUpdateWeather),
                    )
                },
            );
    }

    fn update_weather(&mut self, data: WeatherData) {
        if let Some(ui) = self.app.upgrade() {
            let home_app = ui.global::<HomeViewModel>();
            home_app.set_weather(data);
        }
    }
}

impl App for HomePageApp {
    fn app_name(&self) -> AppName {
        AppName::HomePage
    }

    fn handle_message(
        &mut self,
        ctx: Box<dyn Context>,
        _from: AppName,
        _to: MessageTo,
        msg: Message,
    ) {
        let ctx = Rc::new(ctx);
        match msg {
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Init => {
                    self.init(ctx);
                }
                _ => {}
            },
            Message::HomePage(msg) => match msg {
                HomeMessage::UpdateWeather(data) => {
                    self.update_weather(data);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
