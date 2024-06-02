use std::rc::Rc;

use proto::TopicName;
use slint::{ComponentHandle, Model};

use crate::get_app_window;
use crate::proto::{
    Context, HandleResult, LifecycleMessage, Message, MessageWithHeader, Node, NodeName,
    OneButtonMessage, RouterMessage,
};
use crate::{adapter, ui::MenuViewModel};

pub struct MenuPage {}

impl MenuPage {
    pub fn new() -> Self {
        Self {}
    }

    fn next_page(&self) {
        if let Some(ui) = get_app_window().upgrade() {
            let menu = ui.global::<MenuViewModel>();
            let total_size = menu.get_entry_list().row_count();
            menu.set_current_id((menu.get_current_id() + 1) % total_size as i32);
        }
    }

    fn enter_page(&self, ctx: Rc<dyn Context>) {
        if let Some(ui) = get_app_window().upgrade() {
            let menu = ui.global::<MenuViewModel>();
            if let Some(x) = menu
                .get_entry_list()
                .row_data(menu.get_current_id() as usize)
            {
                ctx.sync_call(
                    NodeName::Router,
                    Message::Router(RouterMessage::GotoPage(
                        adapter::slint_route_table_to_proto_route_table(x.page),
                    )),
                );
            }
        }
    }
}

impl Node for MenuPage {
    fn node_name(&self) -> NodeName {
        NodeName::MenuPage
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Show => {
                    ctx.subscribe_topic(TopicName::OneButton);
                    return HandleResult::Finish(Message::Empty);
                }
                LifecycleMessage::Hide => {
                    ctx.unsubscribe_topic(TopicName::OneButton);
                    return HandleResult::Finish(Message::Empty);
                }
                _ => {}
            },
            Message::OneButton(msg) => match msg {
                OneButtonMessage::Click => {
                    self.next_page();
                    return HandleResult::Finish(Message::Empty);
                }
                OneButtonMessage::Clicks(2) => {
                    self.enter_page(ctx.clone());
                    return HandleResult::Finish(Message::Empty);
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
