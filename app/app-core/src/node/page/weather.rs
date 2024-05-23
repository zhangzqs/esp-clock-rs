use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::proto::{
    Context, HandleResult, LifecycleMessage, Message, MessageTo, MessageWithHeader, Node, NodeName,
    OneButtonMessage, RoutePage, RouterMessage, WeatherMessage,
};
use log::{error, info};

pub struct WeatherPage {
    is_show: RefCell<bool>,
    hold_close_once_flag: RefCell<bool>,
}

impl WeatherPage {
    pub fn new() -> Self {
        Self {
            is_show: RefCell::new(false),
            hold_close_once_flag: RefCell::new(false),
        }
    }
}

impl Node for WeatherPage {
    fn node_name(&self) -> NodeName {
        NodeName::WeatherPage
    }

    fn handle_message(
        &self,
        ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        match msg.body {
            Message::OneButton(msg) => match msg {
                OneButtonMessage::Click => {
                    if *self.is_show.borrow() {
                        ctx.async_call(
                            NodeName::WeatherClient,
                            Message::Weather(WeatherMessage::GetNextSevenDaysWeatherRequest),
                            Box::new(|r| match r {
                                HandleResult::Finish(msg) => {
                                    if let Message::Weather(
                                        WeatherMessage::GetNextSevenDaysWeatherResponse(resp),
                                    ) = msg
                                    {
                                        info!("weather: {:?}", resp);
                                    } else {
                                        error!("weather: {:?}", msg);
                                    }
                                }
                                _ => {}
                            }),
                        );
                    }
                }
                OneButtonMessage::LongPressHolding(dur) => {
                    if !*self.hold_close_once_flag.borrow()
                        && dur > Duration::from_secs(1)
                        && *self.is_show.borrow()
                    {
                        *self.hold_close_once_flag.borrow_mut() = true;
                        ctx.sync_call(
                            NodeName::Router,
                            Message::Router(RouterMessage::GotoPage(RoutePage::Home)),
                        );
                        return HandleResult::Finish(Message::Empty);
                    }
                }
                OneButtonMessage::LongPressHeld(_) => {
                    *self.hold_close_once_flag.borrow_mut() = false
                }
                _ => {}
            },
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Hide => *self.is_show.borrow_mut() = false,
                LifecycleMessage::Show => *self.is_show.borrow_mut() = true,
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
