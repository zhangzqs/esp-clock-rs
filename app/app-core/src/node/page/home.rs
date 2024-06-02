use crate::proto::{
    ipc, Context, HandleResult, LifecycleMessage, Message, MessageWithHeader, Node, NodeName,
    OneButtonMessage, RoutePage, RouterMessage,
};
use crate::ui::{HomeViewModel, TimeData, WeatherData};
use crate::{get_app_window, ui};
use log::{error, info};
use slint::ComponentHandle;
use std::cell::RefCell;
use std::{rc::Rc, time::Duration};
use time::{OffsetDateTime, UtcOffset};

pub struct HomePage {
    time_update_timer: RefCell<Option<slint::Timer>>,
    weather_update_timer: RefCell<Option<slint::Timer>>,
    is_show: RefCell<bool>,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            time_update_timer: RefCell::new(None),
            weather_update_timer: RefCell::new(None),
            is_show: RefCell::new(false),
        }
    }
}

impl HomePage {
    fn update_time(ctx: Rc<dyn Context>) {
        let t = ipc::TimestampClient(ctx).get_timestamp_nanos();
        let t = OffsetDateTime::from_unix_timestamp_nanos(t)
            .unwrap()
            .to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());

        if let Some(ui) = get_app_window().upgrade() {
            let home_app = ui.global::<HomeViewModel>();
            home_app.set_time(TimeData {
                day: t.day() as _,
                hour: t.hour() as _,
                minute: t.minute() as _,
                month: t.month() as _,
                second: t.second() as _,
                week: t.weekday().number_days_from_sunday() as _,
                year: t.year(),
            });
        }
    }

    fn update_weather(ctx: Rc<dyn Context>) {
        ipc::WeatherClient(ctx).get_now_weather(Box::new(|r| match r {
            Ok(x) => {
                if let Some(ui) = get_app_window().upgrade() {
                    let home_app = ui.global::<HomeViewModel>();
                    let w = &x.data;
                    home_app.set_weather(WeatherData {
                        current_humi: w.humidity as _,
                        current_temp: w.now_temperature.round() as _,
                        location: x.city.into(),
                        max_temp: w.max_temperature.round() as _,
                        min_temp: w.min_temperature.round() as _,
                        weather: match w.state {
                            proto::WeatherState::Snow => ui::WeatherState::Snow,
                            proto::WeatherState::Thunder => ui::WeatherState::Thunder,
                            proto::WeatherState::Sandstorm => ui::WeatherState::Sandstorm,
                            proto::WeatherState::Fog => ui::WeatherState::Fog,
                            proto::WeatherState::Hail => ui::WeatherState::Hail,
                            proto::WeatherState::Cloudy => ui::WeatherState::Cloudy,
                            proto::WeatherState::Rain => ui::WeatherState::Rain,
                            proto::WeatherState::Overcast => ui::WeatherState::Overcast,
                            proto::WeatherState::Sunny => ui::WeatherState::Sunny,
                        },
                        air_quality_index: w.air_quality_index as _,
                        air_level: match w.get_air_level() {
                            proto::AirLevel::Good => ui::AirLevel::Good,
                            proto::AirLevel::Moderate => ui::AirLevel::Moderate,
                            proto::AirLevel::UnhealthyForSensitiveGroups => {
                                ui::AirLevel::UnhealthyForSensitiveGroups
                            }
                            proto::AirLevel::Unhealthy => ui::AirLevel::Unhealthy,
                            proto::AirLevel::VeryUnhealthy => ui::AirLevel::VeryUnhealthy,
                            proto::AirLevel::Hazardous => ui::AirLevel::Hazardous,
                        },
                    });
                }
            }
            Err(e) => {
                error!("error: {e:?}");
            }
        }));
    }

    fn on_show(&self, ctx: Rc<dyn Context>) {
        Self::update_time(ctx.clone());
        Self::update_weather(ctx.clone());
        self.time_update_timer
            .borrow_mut()
            .get_or_insert(slint::Timer::default())
            .start(slint::TimerMode::Repeated, Duration::from_secs(1), {
                let ctx = ctx.clone();
                move || {
                    Self::update_time(ctx.clone());
                }
            });
        self.weather_update_timer
            .borrow_mut()
            .get_or_insert(slint::Timer::default())
            .start(slint::TimerMode::Repeated, Duration::from_secs(60), {
                let ctx = ctx.clone();
                move || {
                    Self::update_weather(ctx.clone());
                }
            });
    }

    fn on_hide(&self) {
        self.time_update_timer.borrow_mut().take();
        self.weather_update_timer.borrow_mut().take();
    }
}

impl Node for HomePage {
    fn node_name(&self) -> NodeName {
        NodeName::HomePage
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Show => {
                    *self.is_show.borrow_mut() = true;
                    self.on_show(ctx);
                    return HandleResult::Finish(Message::Empty);
                }
                LifecycleMessage::Hide => {
                    *self.is_show.borrow_mut() = false;
                    self.on_hide();
                    return HandleResult::Finish(Message::Empty);
                }
                _ => {}
            },
            Message::OneButton(msg) => {
                if *self.is_show.borrow() {
                    match msg {
                        OneButtonMessage::Click => {
                            ctx.sync_call(
                                NodeName::Router,
                                Message::Router(RouterMessage::GotoPage(RoutePage::Menu)),
                            );
                            return HandleResult::Finish(Message::Empty);
                        }
                        OneButtonMessage::Clicks(2) => {
                            static mid: &[u8] = include_bytes!("../../../a.mid");
                            ipc::MidiPlayerClient(ctx.clone()).play(
                                mid.to_vec(),
                                Box::new(|r| {
                                    info!("midi播放完毕: {:?}", r);
                                }),
                            );
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
