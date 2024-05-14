use slint::{ComponentHandle, Weak};

use crate::app::{AppWindow, PageRouteTable, PageRouter};
use crate::common::{App, AppName, Context, LifecycleMessage, Message, MessageTo};

pub struct RouterApp {
    app: Weak<AppWindow>,
}

impl RouterApp {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self { app }
    }
    fn goto_page(&self, r: PageRouteTable) {
        if let Some(ui) = self.app.upgrade() {
            let router = ui.global::<PageRouter>();
            router.set_current_page(r);
        }
    }

    fn get_current_page(&self) -> Option<PageRouteTable> {
        self.app
            .upgrade()
            .map(|x| x.global::<PageRouter>().get_current_page())
    }

    fn route_table_to_app_name(r: PageRouteTable) -> AppName {
        match r {
            PageRouteTable::Home => AppName::HomePage,
            PageRouteTable::Menu => AppName::MenuPage,
            PageRouteTable::Weather => AppName::WeatherPage,
        }
    }
}

impl App for RouterApp {
    fn app_name(&self) -> AppName {
        AppName::MenuPage
    }

    fn handle_message(
        &mut self,
        ctx: Box<dyn Context>,
        _from: AppName,
        _to: MessageTo,
        msg: Message,
    ) {
        match msg {
            Message::Router(r) => {
                if let Some(c) = self.get_current_page() {
                    ctx.send_message(
                        MessageTo::App(Self::route_table_to_app_name(c)),
                        Message::Lifecycle(LifecycleMessage::Hide),
                    )
                }
                ctx.send_message(
                    MessageTo::App(Self::route_table_to_app_name(r)),
                    Message::Lifecycle(LifecycleMessage::Show),
                );
                self.goto_page(r);
            }
            _ => {}
        }
    }
}
