use std::time::Duration;

use slint::Weak;

use crate::ui::{AppWindow, PageRouteTable};
use crate::common::{App, AppName, Context, HandleResult, LifecycleMessage, Message, MessageTo};

pub struct BootPageApp {
    app: Weak<AppWindow>,
}

impl BootPageApp {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self { app }
    }
}

impl App for BootPageApp {
    fn app_name(&self) -> AppName {
        AppName::BootPage
    }

    fn handle_message(
        &mut self,
        ctx: Box<dyn Context>,
        _from: AppName,
        _to: MessageTo,
        msg: Message,
    ) -> HandleResult {
        match msg {
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Init => {
                    slint::Timer::single_shot(Duration::from_secs(1), move || {
                        ctx.send_message(
                            MessageTo::App(AppName::Router),
                            Message::Router(PageRouteTable::Home),
                        );
                    });
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
