use slint::{ComponentHandle, Weak};

use crate::common::{App, AppName, Context, HandleResult, LifecycleMessage, Message, MessageTo};
use crate::ui::{AppWindow, PageRouteTable, PageRouter};

pub struct RouterApp {
    app: Weak<AppWindow>,
}

impl RouterApp {
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

    fn route_table_to_app_name(r: PageRouteTable) -> AppName {
        match r {
            PageRouteTable::Boot => AppName::BootPage,
            PageRouteTable::Home => AppName::HomePage,
            PageRouteTable::Menu => AppName::MenuPage,
            PageRouteTable::Weather => AppName::WeatherPage,
        }
    }
}

impl App for RouterApp {
    fn app_name(&self) -> AppName {
        AppName::Router
    }

    fn handle_message(
        &mut self,
        ctx: Box<dyn Context>,
        _from: AppName,
        _to: MessageTo,
        msg: Message,
    ) -> HandleResult {
        match msg {
            Message::Router(r) => {
                if let Some(c) = self.get_current_page() {
                    ctx.send_message(
                        MessageTo::App(Self::route_table_to_app_name(c)),
                        Message::Lifecycle(LifecycleMessage::Hide),
                    )
                }
                let app = self.app.clone();
                ctx.send_message_with_reply_once(
                    MessageTo::App(Self::route_table_to_app_name(r)),
                    Message::Lifecycle(LifecycleMessage::Show),
                    Box::new(move |_, _msg| {
                        Self::goto_page(app, r);
                    }),
                );
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
