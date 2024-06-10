use std::rc::Rc;

use ipc::WeatherClient;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{proto::*, ui};

pub struct WeatherPage {}

impl WeatherPage {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for WeatherPage {
    fn node_name(&self) -> NodeName {
        NodeName::WeatherPage
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::OneButton(msg) => match msg {
                OneButtonMessage::Click => {}
                OneButtonMessage::LongPressHolding(dur) => {
                    if dur > 1000 {
                        ctx.sync_call(
                            NodeName::Router,
                            Message::Router(RouterMessage::GotoPage(RoutePage::Home)),
                        );
                        return HandleResult::Finish(Message::Empty);
                    }
                }
                OneButtonMessage::LongPressHeld(_) => {}
                _ => {}
            },
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Hide => {
                    ctx.unsubscribe_topic(TopicName::OneButton);
                    if let Some(ui) = ui::get_app_window().upgrade() {
                        let vm = ui.global::<ui::WeatherPageViewModel>();
                        vm.set_data(Default::default()); // 释放内存占用
                    }
                }
                LifecycleMessage::Show => {
                    ctx.subscribe_topic(TopicName::OneButton);
                    WeatherClient(ctx.clone()).get_forecast_weather(Box::new(|w| {
                        let data = w
                            .unwrap()
                            .daily
                            .into_iter()
                            .map(|x| ui::OneDayWeatherViewModel {
                                title: {
                                    ["Sun.", "Mon.", "Tue.", "Wed.", "Thu.", "Fri.", "Sat."]
                                        [x.date.weekday().number_from_sunday() as usize]
                                        .into()
                                },
                                date: format!("{:0>2}-{:0>2}", x.date.month() as u8, x.date.day())
                                    .into(),
                                day_icon: x.icon_day as _,
                                day_temp: x.max_temp as _,
                                day_text: x.text_day.into(),
                                night_icon: x.icon_night as _,
                                night_temp: x.min_temp as _,
                                night_text: x.text_night.into(),
                            })
                            .collect::<Vec<_>>();
                        if let Some(ui) = ui::get_app_window().upgrade() {
                            let vm = ui.global::<ui::WeatherPageViewModel>();
                            vm.set_data(ModelRc::new(VecModel::from(data)));
                        }
                    }));
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
