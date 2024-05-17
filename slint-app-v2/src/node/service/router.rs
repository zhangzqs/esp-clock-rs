use slint::{ComponentHandle, Weak};

use crate::proto::{Context, HandleResult, LifecycleMessage, Message, MessageTo, Node, NodeName};
use crate::ui::{AppWindow, PageRouteTable, PageRouter};

pub struct RouterService {
    app: Weak<AppWindow>,
}

impl RouterService {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self { app }
    }
    fn goto_page(app: Weak<AppWindow>, r: PageRouteTable) {
        if let Some(ui) = app.upgrade() {
            let router = ui.global::<PageRouter>();
            router.set_current_page(r);
        }
    }

    fn get_current_page(&self) -> Option<PageRouteTable> {
        self.app
            .upgrade()
            .map(|x| x.global::<PageRouter>().get_current_page())
    }

    fn route_table_to_app_name(r: PageRouteTable) -> NodeName {
        match r {
            PageRouteTable::Boot => NodeName::BootPage,
            PageRouteTable::Home => NodeName::HomePage,
            PageRouteTable::Menu => NodeName::MenuPage,
            PageRouteTable::Weather => NodeName::WeatherPage,
        }
    }
}

impl Node for RouterService {
    fn node_name(&self) -> NodeName {
        NodeName::Router
    }

    fn handle_message(
        &mut self,
        ctx: Box<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: Message,
    ) -> HandleResult {
        match msg {
            Message::Router(r) => {
                if let Some(c) = self.get_current_page() {
                    ctx.send_message(
                        MessageTo::Point(Self::route_table_to_app_name(c)),
                        Message::Lifecycle(LifecycleMessage::Hide),
                    )
                }
                let app = self.app.clone();
                ctx.send_message_with_reply_once(
                    MessageTo::Point(Self::route_table_to_app_name(r)),
                    Message::Lifecycle(LifecycleMessage::Show),
                    Box::new(move |_, _msg| {
                        Self::goto_page(app, r);
                    }),
                );
                return HandleResult::Successful(Message::Empty);
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
