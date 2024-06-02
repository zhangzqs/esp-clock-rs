use crate::proto::*;
use std::rc::Rc;

pub struct MusicPage {}

impl MusicPage {
    pub fn new() -> Self {
        Self {}
    }

    fn on_show() {}
}

impl Node for MusicPage {
    fn node_name(&self) -> NodeName {
        NodeName::MusicPage
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(msg) => match msg {
                // LifecycleMessage::Init => todo!(),
                LifecycleMessage::Show => {
                    ctx.subscribe_topic(TopicName::OneButton);
                }
                LifecycleMessage::Hide => {
                    ctx.unsubscribe_topic(TopicName::OneButton);
                }
                _ => {}
            },
            Message::OneButton(msg) => match msg {
                OneButtonMessage::Click => {}
                OneButtonMessage::LongPressHolding(dur) => {
                    if dur > 3000 {
                        ctx.sync_call(
                            NodeName::Router,
                            Message::Router(RouterMessage::GotoPage(RoutePage::Home)),
                        );
                        return HandleResult::Block;
                    }
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
