use crate::common::{App, AppName, Context, Message, MessageTo};

pub struct HomeApp {}

impl HomeApp {
    pub fn new() -> Self {
        Self {}
    }
}

impl App for HomeApp {
    fn app_name(&self) -> AppName {
        AppName::Home
    }

    fn handle_message(&self, ctx: Box<dyn Context>, from: AppName, to: MessageTo, msg: Message) {
        ctx.send_message(MessageTo::App(AppName::Weather), Message::HomeMessage);
    }
}
