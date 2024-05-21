use std::{rc::Rc, sync::Arc, time::Duration};

use log::info;
use proto::{
    Context, HandleResult, HttpBody, HttpMessage, HttpRequest, HttpRequestMethod, LifecycleMessage,
    Message, MessageTo, MessageWithHeader, Node, NodeName, OneButtonMessage, RoutePage,
    RouterMessage, WeatherMessage,
};

pub struct WeatherPage {
    is_show: bool,
    hold_close_once_flag: bool,
}

impl WeatherPage {
    pub fn new() -> Self {
        Self {
            is_show: false,
            hold_close_once_flag: false,
        }
    }
}

impl Node for WeatherPage {
    fn node_name(&self) -> NodeName {
        NodeName::WeatherPage
    }

    fn handle_message(
        &mut self,
        ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        match msg.body {
            Message::OneButton(msg) => match msg {
                OneButtonMessage::Click => {
                    if self.is_show {
                        ctx.send_message_with_reply_once(
                            MessageTo::Point(NodeName::WeatherClient),
                            Message::Weather(WeatherMessage::GetNextSevenDaysWeatherRequest),
                            Box::new(|_, r| match r {
                                HandleResult::Successful(msg) => {
                                    if let Message::Weather(
                                        WeatherMessage::GetNextSevenDaysWeatherResponse(resp),
                                    ) = msg
                                    {
                                        info!("weather: {:?}", resp);
                                    }
                                }
                                _ => {}
                            }),
                        );
                    }
                }
                OneButtonMessage::LongPressHolding(dur) => {
                    if !self.hold_close_once_flag && dur > Duration::from_secs(1) && self.is_show {
                        self.hold_close_once_flag = true;
                        ctx.send_message(
                            MessageTo::Point(NodeName::Router),
                            Message::Router(RouterMessage::GotoPage(RoutePage::Home)),
                        );
                        return HandleResult::Successful(Message::Empty);
                    }
                }
                OneButtonMessage::LongPressHeld(_) => self.hold_close_once_flag = false,
                _ => {}
            },
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Hide => self.is_show = false,
                LifecycleMessage::Show => self.is_show = true,
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
