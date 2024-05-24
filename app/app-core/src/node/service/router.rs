use std::rc::Rc;

use slint::ComponentHandle;

use crate::get_app_window;
use crate::proto::{
    Context, HandleResult, LifecycleMessage, Message, MessageTo, MessageWithHeader, Node, NodeName,
    RouterMessage,
};
use crate::{adapter, proto::RoutePage, ui::PageRouter};

pub struct RouterService {}

impl RouterService {
    pub fn new() -> Self {
        Self {}
    }
    fn goto_page(r: RoutePage) {
        if let Some(ui) = get_app_window().upgrade() {
            let slint_route = adapter::proto_route_table_to_slint_route_table(r);
            let router = ui.global::<PageRouter>();
            router.set_current_page(slint_route);
        }
    }

    fn get_current_page() -> RoutePage {
        let slint_route = get_app_window()
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
        &self,
        ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        match msg.body {
            Message::Router(RouterMessage::GotoPage(r)) => {
                ctx.sync_call(
                    Self::get_current_page().map_to_node_name(),
                    Message::Lifecycle(LifecycleMessage::Hide),
                );
                ctx.sync_call(
                    r.map_to_node_name(),
                    Message::Lifecycle(LifecycleMessage::Show),
                );
                Self::goto_page(r);
                return HandleResult::Finish(Message::Empty);
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
