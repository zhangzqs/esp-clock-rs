use crate::get_app_window;
use crate::proto::*;
use crate::ui;
use log::error;
use log::info;
use proto::TopicName;
use slint::Color;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::fmt::Debug;
use std::{rc::Rc, time::Duration};
use time::{OffsetDateTime, UtcOffset};

pub struct HomePage {
    time_update_timer: RefCell<Option<slint::Timer>>,
    weather_update_timer: RefCell<Option<slint::Timer>>,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            time_update_timer: RefCell::new(None),
            weather_update_timer: RefCell::new(None),
        }
    }
}

impl HomePage {
    fn update_time() {
        let t = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
        if let Some(ui) = get_app_window().upgrade() {
            let home_app = ui.global::<ui::HomeViewModel>();
            home_app.set_time(ui::TimeData {
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

    fn alert_dialog<T: Debug>(ctx: Rc<dyn Context>, e: T) {
        error!("error: {e:?}");
        ctx.async_call(
            NodeName::AlertDialog,
            Message::AlertDialog(AlertDialogMessage::ShowRequest {
                duration: Some(3000),
                content: AlertDialogContent {
                    text: Some(format!("{e:?}")),
                    image: None,
                },
            }),
            Box::new(|_| {}),
        )
    }

    fn update_weather(ctx: Rc<dyn Context>) {
        let update_ui = |forecast: ForecastWeather,
                         now_weather: NowWeather,
                         now_air_quality: NowAirQuality,
                         location: Location| {
            if let Some(ui) = get_app_window().upgrade() {
                let home_app = ui.global::<ui::HomeViewModel>();
                home_app.set_weather(ui::WeatherData {
                    location: location.location.into(),
                    current_humi: now_weather.humidity as _,
                    current_temp: now_weather.temp as _,
                    weather: now_weather.text.into(),
                    icon: now_weather.icon as _,
                    min_temp: forecast.daily[0].min_temp as _,
                    max_temp: forecast.daily[0].max_temp as _,
                    air_quality_color: Color::from_rgb_u8(
                        now_air_quality.color.0,
                        now_air_quality.color.1,
                        now_air_quality.color.2,
                    ),
                    air_quality_index: now_air_quality.value as _,
                    air_quality_text: now_air_quality.category.into(),
                });
            }
        };

        type RefValue<T> = Rc<RefCell<Option<Result<T, WeatherError>>>>;

        let wg = ctx.create_wait_group();
        let now_weather = RefValue::<NowWeather>::default();
        ipc::WeatherClient(ctx.clone()).get_now_weather(Box::new({
            let wg = wg.clone();
            wg.inc();
            let now_weather = now_weather.clone();
            move |r| {
                *now_weather.borrow_mut() = Some(r);
                wg.done();
            }
        }));

        let forecast_weather = RefValue::<ForecastWeather>::default();
        ipc::WeatherClient(ctx.clone()).get_forecast_weather(Box::new({
            let wg = wg.clone();
            wg.inc();
            let forecast_weather = forecast_weather.clone();
            move |r| {
                *forecast_weather.borrow_mut() = Some(r);
                wg.done();
            }
        }));

        let now_air_quality = RefValue::<NowAirQuality>::default();
        ipc::WeatherClient(ctx.clone()).get_now_air_quality(Box::new({
            let wg = wg.clone();
            wg.inc();
            let now_air_quality = now_air_quality.clone();
            move |r| {
                *now_air_quality.borrow_mut() = Some(r);
                wg.done();
            }
        }));

        let location = match ipc::WeatherClient(ctx.clone()).get_location() {
            Ok(x) => x,
            Err(e) => {
                Self::alert_dialog(ctx.clone(), e);
                return;
            }
        };
        wg.wait(Box::new(move || {
            let f = || -> Result<(), WeatherError> {
                let now_weather = now_weather.borrow_mut().take().unwrap()?;
                let forecast_weather = forecast_weather.borrow_mut().take().unwrap()?;
                let now_air_quality = now_air_quality.borrow_mut().take().unwrap()?;
                update_ui(forecast_weather, now_weather, now_air_quality, location);
                Ok(())
            };
            if let Err(e) = f() {
                Self::alert_dialog(ctx, e);
            }
        }));
    }

    fn on_show(&self, ctx: Rc<dyn Context>) {
        Self::update_time();
        Self::update_weather(ctx.clone());
        self.time_update_timer
            .borrow_mut()
            .get_or_insert(slint::Timer::default())
            .start(
                slint::TimerMode::Repeated,
                Duration::from_secs(1),
                move || {
                    Self::update_time();
                },
            );
        self.weather_update_timer
            .borrow_mut()
            .get_or_insert(slint::Timer::default())
            .start(
                slint::TimerMode::Repeated,
                Duration::from_secs(60),
                move || {
                    Self::update_weather(ctx.clone());
                },
            );
    }

    fn on_hide(&self) {
        self.time_update_timer.borrow_mut().take();
        self.weather_update_timer.borrow_mut().take();
        if let Some(ui) = ui::get_app_window().upgrade() {
            let vm = ui.global::<ui::HomeViewModel>();
            vm.set_weather(Default::default());
            vm.set_time(Default::default());
        }
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
                    ctx.subscribe_topic(TopicName::OneButton);
                    self.on_show(ctx);
                    return HandleResult::Finish(Message::Empty);
                }
                LifecycleMessage::Hide => {
                    ctx.unsubscribe_topic(TopicName::OneButton);
                    self.on_hide();
                    return HandleResult::Finish(Message::Empty);
                }
                _ => {}
            },
            Message::OneButton(msg) => match msg {
                OneButtonMessage::Click => {
                    ctx.sync_call(
                        NodeName::Router,
                        Message::Router(RouterMessage::GotoPage(RoutePage::Menu)),
                    );
                    return HandleResult::Finish(Message::Empty);
                }
                OneButtonMessage::Clicks(2) => {
                    static MID: &[u8] = include_bytes!("../../../a.mid");
                    ipc::MidiPlayerClient(ctx.clone()).play(
                        MID.to_vec(),
                        Box::new(|r| {
                            info!("midi播放完毕: {:?}", r);
                        }),
                    );
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
