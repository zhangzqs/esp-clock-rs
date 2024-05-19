use std::{rc::Rc, time::Duration};

use slint::Weak;

use crate::ui::{AppWindow, PageRouteTable};
use proto::{
    Context, HandleResult, LifecycleMessage, Message, MessageTo, Node, NodeName, RoutePage,
    RouterMessage,
};

pub struct BootPage {
    app: Weak<AppWindow>,
}

impl BootPage {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self { app }
    }
}

impl Node for BootPage {
    fn node_name(&self) -> NodeName {
        NodeName::BootPage
    }

    fn handle_message(
        &mut self,
        ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: Message,
    ) -> HandleResult {
        match msg {
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Init => {
                    slint::Timer::single_shot(Duration::from_secs(1), move || {
                        ctx.send_message(
                            MessageTo::Point(NodeName::Router),
                            Message::Router(RouterMessage::GotoPage(RoutePage::Home)),
                        );
                    });
                    return HandleResult::Successful(Message::Empty);
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
