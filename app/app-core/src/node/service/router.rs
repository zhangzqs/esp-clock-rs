use std::rc::Rc;

use slint::{ComponentHandle, Weak};

use crate::{
    adapter::{self},
    ui::{AppWindow, PageRouter},
};
use proto::{
    Context, HandleResult, LifecycleMessage, Message, MessageTo, MessageWithHeader, Node, NodeName,
    RouterMessage,
};

pub struct RouterService {
    app: Weak<AppWindow>,
}

impl RouterService {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self { app }
    }
    fn goto_page(app: Weak<AppWindow>, r: proto::RoutePage) {
        if let Some(ui) = app.upgrade() {
            let slint_route = adapter::proto_route_table_to_slint_route_table(r);
            let router = ui.global::<PageRouter>();
            router.set_current_page(slint_route);
        }
    }

    fn get_current_page(&self) -> proto::RoutePage {
        let slint_route = self
            .app
            .upgrade()
            .unwrap()
            .global::<PageRouter>()
            .get_current_page();
        adapter::slint_route_table_to_proto_route_table(slint_route)
    }
}

impl Node for RouterService {
    fn node_name(&self) -> NodeName {
        NodeName::Router
    }

    fn handle_message(
        &mut self,
        ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        match msg.body {
            Message::Router(RouterMessage::GotoPage(r)) => {
                ctx.send_message(
                    MessageTo::Point(self.get_current_page().map_to_node_name()),
                    Message::Lifecycle(LifecycleMessage::Hide),
                );
                let app = self.app.clone();
                ctx.send_message_with_reply_once(
                    MessageTo::Point(r.map_to_node_name()),
                    Message::Lifecycle(LifecycleMessage::Show),
                    Box::new(move |_, _msg| Self::goto_page(app, r)),
                );
                return HandleResult::Successful(Message::Empty);
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
