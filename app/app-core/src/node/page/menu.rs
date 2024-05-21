use std::rc::Rc;

use slint::{ComponentHandle, Model, Weak};

use crate::{
    adapter,
    ui::{AppWindow, MenuViewModel},
};
use proto::{
    Context, HandleResult, LifecycleMessage, Message, MessageTo, MessageWithHeader, Node, NodeName, OneButtonMessage, RouterMessage
};

pub struct MenuPage {
    app: Weak<AppWindow>,
    is_show: bool,
}

impl MenuPage {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self {
            app,
            is_show: false,
        }
    }

    fn next_page(&self) {
        if let Some(ui) = self.app.upgrade() {
            let menu = ui.global::<MenuViewModel>();
            let total_size = menu.get_entry_list().row_count();
            menu.set_current_id((menu.get_current_id() + 1) % total_size as i32);
        }
    }

    fn enter_page(&self, ctx: Rc<dyn Context>) {
        if let Some(ui) = self.app.upgrade() {
            let menu = ui.global::<MenuViewModel>();
            if let Some(x) = menu
                .get_entry_list()
                .row_data(menu.get_current_id() as usize)
            {
                ctx.send_message(
                    MessageTo::Point(NodeName::Router),
                    Message::Router(RouterMessage::GotoPage(
                        adapter::slint_route_table_to_proto_route_table(x.page),
                    )),
                )
            }
        }
    }
}

impl Node for MenuPage {
    fn node_name(&self) -> NodeName {
        NodeName::MenuPage
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
                    return HandleResult::Successful(Message::Empty);
                }
                LifecycleMessage::Hide => {
                    self.is_show = false;
                    return HandleResult::Successful(Message::Empty);
                }
                _ => {}
            },
            Message::OneButton(msg) => {
                if self.is_show {
                    match msg {
                        OneButtonMessage::Click => {
                            self.next_page();
                            return HandleResult::Successful(Message::Empty);
                        }
                        OneButtonMessage::Clicks(2) => {
                            self.enter_page(ctx.clone());
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
