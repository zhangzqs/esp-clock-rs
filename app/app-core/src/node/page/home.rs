use crate::ui::{AppWindow, HomeViewModel, TimeData, WeatherData};
use proto::{
    Context, HandleResult, LifecycleMessage, Message, MessageTo, MessageWithHeader, Node, NodeName, OneButtonMessage, RoutePage, RouterMessage
};
use slint::{ComponentHandle, Weak};
use std::{rc::Rc, time::Duration};
use time::{OffsetDateTime, UtcOffset};

pub struct HomePage {
    app: Weak<AppWindow>,
    time_update_timer: Option<slint::Timer>,
    weather_update_timer: Option<slint::Timer>,
    is_show: bool,
}

impl HomePage {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self {
            app,
            time_update_timer: None,
            weather_update_timer: None,
            is_show: false,
        }
    }
}

impl HomePage {
    fn update_time(app: Weak<AppWindow>, ctx: Rc<dyn Context>) {
        proto::ipc::get_timestamp_nanos(
            ctx,
            Box::new(move |t| {
                let t = OffsetDateTime::from_unix_timestamp_nanos(t)
                    .unwrap()
                    .to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
                if let Some(ui) = app.upgrade() {
                    let home_app = ui.global::<HomeViewModel>();
                    home_app.set_time(TimeData {
                        day: t.day() as _,
                        hour: t.hour() as _,
                        minute: t.minute() as _,
                        month: t.month() as _,
                        second: t.second() as _,
                        week: t.weekday().number_days_from_sunday() as _,
                        year: t.year(),
                    })
                }
            }),
        )
    }

    fn on_show(&mut self, ctx: Rc<dyn Context>) {
        let app = self.app.clone();
        Self::update_time(app.clone(), ctx.clone());

        let ctx_ref = ctx.clone();
        self.time_update_timer
            .get_or_insert(slint::Timer::default())
            .start(
                slint::TimerMode::Repeated,
                Duration::from_secs(1),
                move || {
                    Self::update_time(app.clone(), ctx_ref.clone());
                },
            );
        self.weather_update_timer
            .get_or_insert(slint::Timer::default())
            .start(
                slint::TimerMode::Repeated,
                Duration::from_secs(60),
                move || {
                    // ctx.send_message(
                    //     MessageTo::Point(NodeName::WeatherClient),
                    //     Message::HomePage(HomeMessage::RequestUpdateWeather),
                    // )
                },
            );
    }

    fn on_hide(&mut self) {
        self.time_update_timer.take();
        self.weather_update_timer.take();
    }

    fn update_weather(&mut self, data: WeatherData) {
        if let Some(ui) = self.app.upgrade() {
            let home_app = ui.global::<HomeViewModel>();
            home_app.set_weather(data);
        }
    }
}

impl Node for HomePage {
    fn node_name(&self) -> NodeName {
        NodeName::HomePage
    }

    fn handle_message(
        &mut self,
        ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        match msg.body {
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Show => {
                    self.is_show = true;
                    self.on_show(ctx);
                    return HandleResult::Successful(Message::Empty);
                }
                LifecycleMessage::Hide => {
                    self.is_show = false;
                    self.on_hide();
                    return HandleResult::Successful(Message::Empty);
                }
                _ => {}
            },
            // Message::HomePage(msg) => match msg {
            //     HomeMessage::UpdateWeather(data) => {
            //         self.update_weather(data);
            //         ctx.send_message_with_reply_once(
            //             MessageTo::Point(NodeName::HttpClient),
            //             Message::Http(HttpMessage::Request(Rc::new(HttpRequest {
            //                 method: HttpRequestMethod::Get,
            //                 url: "http://www.baidu.com".to_string(),
            //                 header: None,
            //                 body: HttpBody::Empty,
            //             }))),
            //             Box::new(|n, r| match r {
            //                 HandleResult::Successful(msg) => {
            //                     if let Message::Http(HttpMessage::Response(resp)) = msg {
            //                         if let HttpBody::Bytes(bs) = resp.body.clone() {
            //                             println!("{:?}", String::from_utf8(bs));
            //                         }
            //                     }
            //                 }
            //                 _ => {}
            //             }),
            //         );
            //         return HandleResult::Successful(Message::Empty);
            //     }
            //     _ => {}
            // },
            Message::OneButton(msg) => {
                if self.is_show {
                    match msg {
                        OneButtonMessage::Click => {
                            ctx.send_message(
                                MessageTo::Point(NodeName::Router),
                                Message::Router(RouterMessage::GotoPage(RoutePage::Menu)),
                            );
                            return HandleResult::Successful(Message::Empty);
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
