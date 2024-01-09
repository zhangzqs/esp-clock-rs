use std::{marker::PhantomData, thread, time::Duration};

use client::weather::WeatherClient;

use log::{info};
use slint::Weak;
use time::{OffsetDateTime, UtcOffset};

use crate::{AppWindow, ClientBuilder, HomeTimeData, HomeWeatherData};
pub struct HomeApp<CB> {
    app: Weak<AppWindow>,
    home_time_timer: slint::Timer,
    p: PhantomData<CB>,
}

impl<CB> HomeApp<CB>
where
    CB: ClientBuilder,
{
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self::start_weather_timer(app.clone());
        Self {
            app: app.clone(),
            home_time_timer: Self::start_home_time_timer(app.clone()),
            p: PhantomData,
        }
    }

    fn start_weather_timer(w: Weak<AppWindow>) {
        thread::spawn(move || -> anyhow::Result<()> {
            let mut client = CB::new().build_client().unwrap();
            let mut weather = WeatherClient::new("http://192.168.242.118:3000", &mut client);
            let city_resp = weather.city_lookup::<1024>("Shanghai").unwrap();
            let c = city_resp.items.first().unwrap();
            let now_resp = weather.now::<1024>(&c.id).unwrap();
            info!("weather: {:?}", now_resp);
            w.upgrade_in_event_loop(move |ui| {
                ui.set_home_page_weather(HomeWeatherData {
                    current_temp: now_resp.temp,
                    current_humi: now_resp.humidity as i32,
                    ..Default::default()
                });
            })
            .unwrap();
            Ok(())
        });
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
