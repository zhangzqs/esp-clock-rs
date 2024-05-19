use std::rc::Rc;

use proto::{
    Context, HandleResult, LifecycleMessage, Message, MessageTo, Node, NodeName, OneButtonMessage,
    RoutePage, RouterMessage,
};

pub struct WeatherPage {
    is_show: bool,
}

impl WeatherPage {
    pub fn new() -> Self {
        Self { is_show: false }
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
        msg: Message,
    ) -> HandleResult {
        match msg {
            Message::OneButton(OneButtonMessage::LongPressHolding(_)) => {
                if !self.is_show {
                    return HandleResult::Discard;
                }
                ctx.send_message(
                    MessageTo::Point(NodeName::Router),
                    Message::Router(RouterMessage::GotoPage(RoutePage::Home)),
                );
                return HandleResult::Successful(Message::Empty);
            }
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
